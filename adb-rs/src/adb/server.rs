use std::{fs, io, thread};
use std::fmt::{Display, Write};
use std::io::{Read, Write as stdWrite};
use std::path::Path;
use std::str::FromStr;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

use bytes::{Buf, BufMut, BytesMut};
use bytes::buf::Reader;
use chrono::NaiveDateTime;

use crate::{AdbClient, AdbConfig, Result};
use crate::adb::base_type::{DeviceEvent, DeviceStatus, MappingDevice, MappingType, RemoteFileInfo, ShellResult};
use crate::client::AdbConnState;
use crate::errors::AdbError;

pub struct Transfer<'a> {
    inner: &'a mut AdbClient,
    serial: &'a mut String,
}

impl<'a> Transfer<'a> {
    fn new(dev: &'a mut AdbDevice) -> Self {
        Self {
            inner: &mut dev.inner,
            serial: &mut dev.serial,
        }
    }

    fn ready_to_transfer<S, P>(&mut self, command: S, path: P) -> Result<()>
        where S: Into<String>, P: AsRef<Path> {
        let _cmd = vec!["host", "transport", self.serial.as_str()];
        let cmd: String = _cmd.join(":");
        self.inner.send(cmd.as_bytes())?;
        self.inner.check_ok()?;
        self.inner.send("sync:".as_bytes())?;
        self.inner.check_ok()?;
        // +-------+----------------------+----+
        // |COMMAND|LittleEndianPathLength|Path|
        // +-------+----------------------+----+
        let mut buffer = BytesMut::new();
        buffer.write_str(command.into().as_str())?; // COMMAND
        if let Some(p) = path.as_ref().to_str() {
            let length = u32::to_le_bytes(p.len() as u32);
            let mut writer = buffer.writer();
            writer.write(&length)?; // LittleEndianPathLength
            writer.write(p.as_bytes())?;// Path
            return Ok(());
        }
        Err(AdbError::FilePathErr { path: path.as_ref().to_string_lossy().to_string() })
    }

    /// 获取设备中文件信息，如果文件不存在返回None（待完成）
    pub fn stat<P>(&mut self, path: P) -> Result<Option<RemoteFileInfo>> where P: AsRef<Path> {
        let ph = path.as_ref().to_string_lossy().to_string();
        self.ready_to_transfer("STAT", path)?;
        let mut buffer = self.inner.read_n(12)?;
        let mut reader = buffer.reader();
        let mode = parse_u32(&mut reader)?;
        let size = parse_u32(&mut reader)?;
        let mtime = parse_u32(&mut reader)?;
        //todo: check file exists?
        let file = RemoteFileInfo {
            mode,
            size: size as usize,
            timestamp: NaiveDateTime::from_timestamp(mtime as i64, 0),
            path: ph,
        };
        Ok(Some(file))
    }
    /// 获取设备中文件目录中所有的文件信息
    pub fn ls_dir<P>(&mut self, path: P) -> Result<Vec<RemoteFileInfo>> where P: AsRef<Path> {
        self.ready_to_transfer("LIST", path)?;
        let mut dirs = Vec::new();
        loop {
            if self.inner.check_code_is(AdbConnState::DONE)? {
                break;
            }
            let buffer = self.inner.read_n(16)?;
            let mut reader = buffer.reader();
            let mode = parse_u32(&mut reader)?;
            let size = parse_u32(&mut reader)?;
            let mtime = parse_u32(&mut reader)?;
            let content_len = parse_u32(&mut reader)?;
            let name = AdbClient::read_n_string(&mut self.inner, content_len as u64)?;
            dirs.push(RemoteFileInfo {
                mode,
                size: size as usize,
                timestamp: NaiveDateTime::from_timestamp(mtime as i64, 0),
                path: name,
            })
        }
        Ok(dirs)
    }

    /// 获取设备中文件目录中所有的文件信息
    pub fn push<F: AsRef<Path>>(&mut self, file: F, remote: F) -> Result<usize> {
        let filename = file.as_ref().to_string_lossy().to_string();
        self.ready_to_transfer("SEND", remote)?;
        let mut f = fs::File::open(file)?;
        let mut chunk = f.take(4096);
        let mut buffer = [0u8; 4096];
        let mut total_size = 0;
        while chunk.read(&mut buffer)? > 0 {
            // start transfer file
            self.inner.send(format!("DATA{}", buffer.len()).as_bytes())?;
            let send_size = self.inner.send(buffer.as_ref())?;
            total_size += send_size;
            // check adb server result
            if !self.inner.check_code_is(AdbConnState::OKAY)? {
                return Err(AdbError::FileTransferErr { filename, cause: "unknown".to_owned() });
            }
        }
        Ok(total_size)
    }

    /// 拉取设备中的文件
    pub fn pull<F: AsRef<Path>>(&mut self, file: F, remote: F) -> Result<usize> {
        let filename = file.as_ref().to_string_lossy().to_string();
        self.ready_to_transfer("RECV", remote)?;
        let mut f = fs::File::create(file)?;
        let mut recv_size = 0;
        loop {
            match self.inner.conn_code()? {
                AdbConnState::FAIL => {
                    let msg_len = self.inner.read_u32()?;
                    let msg = AdbClient::read_n_string(self.inner, msg_len as u64)?;
                    return Err(AdbError::FileTransferErr { filename, cause: msg });
                }
                AdbConnState::DONE => {
                    break;
                }
                AdbConnState::DATA => {
                    let chunk_size = self.inner.read_u32()? as u64;
                    let buffer = self.inner.read_n(chunk_size)?;
                    let read_size = buffer.len();
                    if read_size != chunk_size {
                        return Err(AdbError::IoError {
                            why: io::Error::new(
                                io::ErrorKind::UnexpectedEof,
                                format!("read {} bytes but except {} bytes", read_size, chunk_size),
                            )
                        });
                    }
                    recv_size += read_size;
                    f.write(buffer.as_ref())?;
                }

                _ => {}
            }
        }
        Ok(recv_size)
    }
}

fn parse_u32(reader: &mut Reader<BytesMut>) -> std::io::Result<u32> {
    let mut chunk = [0u8; 4];
    reader.read_exact(&mut chunk)?;
    Ok(u32::from_le_bytes(chunk))
}

/// android设备抽象，每个AdbDevice都持有一份AdbClient拷贝
pub struct AdbDevice {
    #[cfg(feature = "server")]
    inner: AdbClient,
    /// 设备序列号
    serial: String,
    /// 设备状态
    status: DeviceStatus,
}

impl AdbDevice {
    fn new(inner: AdbClient, serial: String, status: DeviceStatus) -> Self {
        Self {
            inner,
            serial,
            status,
        }
    }

    /// 向设备进行IO操作
    /// 注意：Transfer中持有self.inner的可变引用，使用时注意生命周期
    pub fn transfer(&mut self) -> Transfer {
        Transfer::new(self)
    }

    /// 在Android设备中执行shell命令
    /// 如果`stream`为true，则返回 `ShellResult::Stream`,否则返回`ShellResult::Output`
    /// 其中`timeout`会同时设置写超时和读超时
    pub fn shell<S>(&mut self, command: S, stream: bool, timeout: Option<Duration>) -> Result<ShellResult> where S: Into<String> + Display {
        let cmd = format!("host:transport:{}", self.serial);
        if stream {
            let mut given_client = self.inner.clone();
            AdbDevice::send_shell(command, &cmd, &mut given_client, timeout)?;
            return Ok(ShellResult::Stream(given_client));
        }
        AdbDevice::send_shell(command, &cmd, &mut self.inner, timeout)?;
        Ok(ShellResult::Output(self.inner.rend_full_string()?))
    }

    fn send_shell<S>(command: S, cmd: &String, given_client: &mut AdbClient, timeout: Option<Duration>) -> Result<()> where S: Into<String> + Display {
        given_client.set_timeout(timeout)?;
        given_client.send(cmd.as_bytes())?;
        given_client.check_ok()?;
        given_client.send(format!("shell:{}", command).as_bytes())?;
        given_client.check_ok()?;
        Ok(())
    }

    /// 获取当前设备的所有转发列表
    pub fn forward_list(&mut self) -> Result<Vec<MappingDevice>> {
        forward_list(&mut self.inner, Some(&self.serial))
    }
    /// 获取当前设备的所有反向代理列表
    pub fn reverse_list(&mut self) -> Result<Vec<MappingDevice>> {
        reverse_list(&mut self.inner, Some(&self.serial))
    }

    /// 与设备建立一个转发，adb -s `self.serial` forward tcp:1000 tcp:2000的意思是，将PC端的1000端口收到的数据，转发给到手机中2000端口
    /// PC可以通过访问自身的2000端口来访问手机的1000端口
    pub fn forward(&mut self, local: u32, remote: MappingType, norebind: Option<bool>) -> Result<()> {
        forward(&mut self.inner, &self.serial, local, remote, norebind)
    }

    /// 与设备建立反向代理，adb -s `self.serial` reverse tcp:1000 tcp:2000的意思是，将手机端的1000端口收到的数据，反向代理到PC中2000端口
    /// 手机可以访问自身的1000端口来访问PC的2000端口
    pub fn reverse(&mut self, remote: u32, local: u32) -> Result<()> {
        reverse(&mut self.inner, &self.serial, remote, local)
    }
}

pub struct AndroidDebugBridge {
    #[cfg(feature = "server")]
    inner: AdbClient,
}

impl AndroidDebugBridge {
    #[cfg(feature = "server")]
    pub fn new(cfg: AdbConfig) -> Result<Self> {
        let client = AdbClient::new(cfg)?;
        Ok(Self {
            inner: client,
        })
    }

    #[cfg(feature = "server")]
    pub fn adb_version(&mut self) -> Result<u32> {
        let res = self.inner.read_string()?;
        let version = u32::from_str(res.as_ref())?;
        Ok(version)
    }

    #[cfg(feature = "server")]
    pub fn kill_server(&mut self) -> Result<()> {
        // todo:check server is online
        self.inner.send("host:kill".as_bytes())?;
        self.inner.check_ok()?;
        Ok(())
    }

    #[cfg(feature = "server")]
    pub fn connect(&mut self, addr: &String) -> Result<()> {
        self.inner.send(format!("host:connect:{}", addr).as_bytes())?;
        self.inner.check_ok()?;
        //todo: check connect result
        self.inner.read_string()?;
        Ok(())
    }

    #[cfg(feature = "server")]
    pub fn disconnect(&mut self, addr: &String) -> Result<String> {
        self.inner.send(format!("host:disconnect:{}", addr).as_bytes())?;
        self.inner.check_ok()?;
        self.inner.read_string()
    }

    #[cfg(feature = "server")]
    pub fn watch_device(&mut self) -> Receiver<DeviceEvent> {
        let (mut sender, receiver) = channel();
        let mut th_client = self.inner.clone();
        thread::spawn(move || {
            AndroidDebugBridge::_watch_device(&mut sender, &mut th_client)
        });
        receiver
    }

    fn _watch_device(receiver: &mut Sender<DeviceEvent>, th_client: &mut AdbClient) -> Result<()> {
        th_client.send("host:track-devices".as_bytes())?;
        th_client.check_ok()?;
        loop {
            let raw_info = th_client.read_string()?;
            let events = raw_info
                .split("\n")
                .map(|line| line.split_ascii_whitespace())
                .fold(Vec::new(), |mut acc, mut item| {
                    let serial = item.next().map(|x| x.to_string());
                    let status = item.next();
                    if serial.is_some() && status.is_some() {
                        acc.push(DeviceEvent {
                            present: None,
                            serial: serial.unwrap(),
                            status: DeviceStatus::from_str(status.unwrap()).unwrap(),
                        })
                    }
                    acc
                });
            //todo: check different?
            for event in events {
                if let Err(_) = receiver.send(event) {
                    break;
                }
            }
        }
    }

    /// 获取当所有反向代理列表
    pub fn reverse_list(&mut self) -> Result<Vec<MappingDevice>> {
        reverse_list(&mut self.inner, None)
    }

    /// 根据serial获取设备
    pub fn find_devices(&mut self, serial: String) -> Result<Option<AdbDevice>> {
        let mut find_result = self.available_devices().and_then(|d| {
            let res = d.into_iter().filter(|dev| dev.serial == serial).collect::<Vec<AdbDevice>>();
            Ok(res)
        })?;
        Ok(find_result.pop())
    }

    /// 获取当所有可用的设备
    pub fn available_devices(&mut self) -> Result<Vec<AdbDevice>> {
        self.devices().and_then(|x| {
            let res = x
                .into_iter()
                .filter(|dev| dev.status == DeviceStatus::Online)
                .collect::<Vec<AdbDevice>>();
            Ok(res)
        })
    }

    /// 获取当所有设备，包括可用和不可用
    pub fn devices(&mut self) -> Result<Vec<AdbDevice>> {
        self.inner.send("host:devices".as_bytes())?;
        self.inner.check_ok()?;
        let device_str = self.inner.read_string()?;
        let devices = device_str
            .split("\n")
            .map(|line| line.split_ascii_whitespace())
            .fold(Vec::new(), |mut acc, mut item| {
                let serial = item.next();
                let status = item.next();
                if serial.is_some() && status.is_some() {
                    let dev = AdbDevice::new(
                        self.inner.clone(),
                        serial.unwrap().to_string(),
                        DeviceStatus::from_str(status.unwrap()).unwrap(),
                    );
                    acc.push(dev);
                }
                acc
            });
        Ok(devices)
    }
}


fn forward_list(client: &mut AdbClient, serial: Option<&String>) -> Result<Vec<MappingDevice>> {
    let cmd = if let Some(s) = serial {
        format!("host-serial:{}:list-forward", s)
    } else {
        String::from("host:list-forward")
    };
    client.send(cmd.as_bytes())?;
    client.check_ok()?;
    // parse result
    let list = client.read_string()?;
    let res = list
        .split("\n")
        .map(|line| line.split_ascii_whitespace())
        .fold(Vec::new(), |mut acc, mut item| {
            // <serial local remote>
            let serial = item.next().map(|data| data.to_string());
            let local = item.next();
            let remote = item.next();
            check_and_push(&mut acc, serial, local, remote);
            acc
        });
    Ok(res)
}

fn check_and_push(acc: &mut Vec<MappingDevice>, serial: Option<String>, local: Option<&str>, remote: Option<&str>) {
    if serial.is_some() && local.is_some() && remote.is_some() {
        acc.push(MappingDevice::new(serial, local.unwrap(), remote.unwrap()));
    }
}

fn forward(client: &mut AdbClient, serial: &String, local: u32, remote: MappingType, norebind: Option<bool>) -> Result<()> {
    let mut cmds = vec!["host-serial", serial.as_ref(), "forward"];
    if norebind.is_some() && norebind.unwrap() {
        cmds.push("norebind")
    }
    // it cloud be better?
    let forward_cmd = match remote {
        MappingType::Ipc(ref ipc) => format!("{};{}", local, ipc),
        MappingType::Port(port) => format!("{};{}", local, port)
    };
    cmds.push(forward_cmd.as_ref());
    let res: String = cmds.join(";");
    client.send(res.as_bytes())?;
    Ok(())
}


fn reverse(client: &mut AdbClient, serial: &String, remote: u32, local: u32) -> Result<()> {
    client.send(format!("{},{}", "host:transport:", serial).as_bytes())?;
    client.check_ok()?;
    let sub_cmd: String = format!("{};{}", remote, local);
    let cmds = vec!["reverse:forward", sub_cmd.as_str()];
    client.send(cmds.join(":").as_bytes())?;
    Ok(())
}

fn reverse_list(client: &mut AdbClient, serial: Option<&String>) -> Result<Vec<MappingDevice>> {
    let cmd = if let Some(s) = serial {
        format!("host:transport:{}", s)
    } else {
        String::from("host:transport")
    };
    client.send(cmd.as_bytes())?;
    client.check_ok()?;
    client.send("reverse:list-forward".as_bytes())?;
    client.check_ok()?;
    let result = client.read_string()?;
    // it possible to reuse `forward_list`?
    let devices = result.split("\n")
        .map(|line| line.split_ascii_whitespace())
        .fold(Vec::new(), |mut acc, mut item| {
            let serial = item.next().map(|data| data.to_string());
            let remote = item.next();
            let local = item.next();
            check_and_push(&mut acc, serial, local, remote);
            acc
        });
    Ok(devices)
}
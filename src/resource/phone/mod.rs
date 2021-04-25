pub mod adb;
pub mod phone;


#[test]
fn test_fn() {
	let res = std::env::current_dir().unwrap();
	println!("{}", res.to_str().unwrap());
}
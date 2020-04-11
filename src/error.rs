#[derive(thiserror::Error, Debug)]
#[error("Fail error occurred.")]
pub struct MyError {}

#[derive(thiserror::Error, Debug)]
pub enum MyBgpError {
    #[error("This is BGP error")]
    BgpError,
}

pub fn error_io_test() -> Result<(), anyhow::Error> {
    let _buf = std::fs::File::open("hoge")?;
    Ok(())
}

pub fn error_my_test() -> Result<(), anyhow::Error> {
    Err(MyError {}.into())
}

pub fn error_my_bgp_test() -> Result<(), anyhow::Error> {
    Err(MyBgpError::BgpError.into())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn downcast_io() {
        if let Err(err) = error_io_test() {
            if let Some(io_error) = err.downcast_ref::<std::io::Error>() {
                println!("IO error: {}", io_error);
            }
            if let Some(_) = err.downcast_ref::<MyError>() {
                panic!("MyError");
            }
            if let Some(_) = err.downcast_ref::<MyBgpError>() {
                panic!("MyBgpError");
            }
        }
    }

    #[test]
    fn downcast_my() {
        if let Err(err) = error_my_test() {
            if let Some(_) = err.downcast_ref::<std::io::Error>() {
                panic!("IO error")
            }
            if let Some(my_error) = err.downcast_ref::<MyError>() {
                println!("MyError: {}", my_error);
            }
            if let Some(_) = err.downcast_ref::<MyBgpError>() {
                panic!("MyBgpError");
            }
        }
    }

    #[test]
    fn downcast_my_bgp() {
        if let Err(err) = error_my_bgp_test() {
            if let Some(_) = err.downcast_ref::<std::io::Error>() {
                panic!("IO error")
            }
            if let Some(_) = err.downcast_ref::<MyError>() {
                panic!("MyError");
            }
            if let Some(my_bgp_error) = err.downcast_ref::<MyBgpError>() {
                println!("MyBgpError: {}", my_bgp_error);
            }
        }
    }
}

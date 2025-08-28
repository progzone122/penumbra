use crate::da::DA;w

pub trait DAProtocol {
    fn upload_da(&mut self, &da: DA) -> Result<bool, String>;
    fn boot_to(&mut self, &da: DA, &addr: u32) -> Result<bool, String>;
    
}
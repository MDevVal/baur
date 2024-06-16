#[derive(Debug)]
pub struct Args {
    pub operation: Option<char>,
    pub operation_flags: Vec<char>,
    pub target: Option<String>,
    pub additional_options: Vec<String>,
}

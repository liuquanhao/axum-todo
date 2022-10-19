#[derive(Debug)]
pub enum TodoRepoError {
    #[allow(dead_code)]
    NotFound,
    #[allow(dead_code)]
    DatabaseError,
}

use uuid::Uuid;

use crate::auth::roles::StudentStatus;
use crate::domain::entities::user_row::UserRow;
use crate::infra::errors::RepoResult;
use crate::infra::repositories::user_repo::UserRepository;

#[derive(Clone)]
pub struct UsersService<R: UserRepository + Send + Sync + 'static> {
    repo: R,
}

impl<R: UserRepository + Send + Sync + 'static> UsersService<R> {
    pub fn new(repo: R) -> Self { Self { repo } }

    pub async fn list_students_by_status(
        &self,
        statuses: &[StudentStatus],
        page: i32,
        limit: i32,
        search: &str,
    ) -> RepoResult<Vec<(UserRow, Option<StudentStatus>)>> {
        self.repo.list_students_by_status(statuses, page, limit, search).await
    }

    pub async fn set_student_status(
        &self,
        user_id: Uuid,
        status: StudentStatus,
    ) -> RepoResult<()> {
        self.repo.set_student_status(user_id, status).await
    }
}
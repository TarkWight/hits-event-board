#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role { Student, Manager, Dean }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagerStatus { Pending, Confirmed, Rejected }
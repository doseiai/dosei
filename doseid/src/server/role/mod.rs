use uuid::Uuid;

#[derive(Debug)]
pub struct Role {
  pub id: Uuid,
  pub name: &'static str,
  pub description: &'static str,
}

macro_rules! roles {
    ($($name:ident: ($id:expr, $desc:expr)),* $(,)?) => {
        #[allow(dead_code)]
        impl Role {
            $(
                pub const $name: Role = Role {
                    id: Uuid::from_bytes($id),
                    name: stringify!($name),
                    description: $desc,
                };
            )*
        }
    }
}

roles! {
    OWNER: (
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    "Has full administrative access to the entire organization."
  ),
    MEMBER: (
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
    "Can see everything in the organization and create new services."
  ),
}

#[cfg(test)]
mod tests {
  use crate::server::role::Role;
  use uuid::Uuid;

  #[test]
  fn test_roles() {
    assert_eq!(
      Role::OWNER.id,
      Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
    );
    assert_eq!(Role::OWNER.name, "OWNER");

    assert_eq!(
      Role::MEMBER.id,
      Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap()
    );
    assert_eq!(Role::MEMBER.name, "MEMBER");
  }
}

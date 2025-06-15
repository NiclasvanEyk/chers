struct User {
    index: usize,
    ready: bool,
    name: Option<String>,
}

#[derive(Default)]
pub struct Lobby {
    users: [Option<User>; 2],
}

impl Lobby {
    pub fn try_join(&mut self) -> Option<u32> {
        if self.users[0].is_none() {
            self.users[0] = Some(User {
                index: 0,
                name: None,
                ready: false,
            });
            return Some(0);
        }

        if self.users[1].is_none() {
            self.users[1] = Some(User {
                index: 1,
                name: None,
                ready: false,
            });
            return Some(1);
        }

        return None;
    }
}

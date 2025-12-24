/*

Plan

- Start chat page
- Chat page
- See current chats page

*/

#[derive(Clone, Copy, Debug)]
pub enum Pages {
    Chat(usize),
    AddChat,
    BrowseChats
}

pub mod chat_page;
pub mod add_chat_page;
pub mod browse_chats_page;

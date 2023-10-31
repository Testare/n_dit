pub enum DialogueType {
    ChatAlert, // Message appears in "chat box"
    Alert, // Pops up for a period of time. Is this really dialogue?
    Menu, // Interactive, but easily left
    Character, // Interactive, but cannot be easily left?
}
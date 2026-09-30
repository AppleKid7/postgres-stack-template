use spacetimedb::{table, reducer, ReducerContext, Timestamp};

#[table(name = message, public)]
pub struct Message {
    #[primary_key]
    #[auto_inc]
    id: u64,
    sender: String,
    text: String,
    created_at: Timestamp,
}

#[reducer]
pub fn send_message(ctx: &ReducerContext, sender: String, text: String) {
    log::info!("New message from {}: {}", sender, text);
    ctx.db.message().insert(Message {
        id: 0,
        sender,
        text,
        created_at: ctx.timestamp,
    });
}

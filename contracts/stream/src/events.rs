use soroban_sdk::{symbol_short, Address, Env};

/// Emitted when a stream contract is initialized by the factory.
pub fn stream_initialized(env: &Env, factory: &Address, sender: &Address, recipient: &Address) {
    env.events()
        .publish((symbol_short!("init"), factory, sender), recipient);
}

/// Emitted when the recipient withdraws accrued funds.
pub fn withdrawn(env: &Env, recipient: &Address, amount: i128) {
    env.events()
        .publish((symbol_short!("withdraw"), recipient), amount);
}

/// Emitted when the sender pauses the stream.
pub fn paused(env: &Env, sender: &Address) {
    env.events().publish((symbol_short!("paused"),), sender);
}

/// Emitted when the sender resumes a paused stream.
pub fn resumed(env: &Env, sender: &Address) {
    env.events().publish((symbol_short!("resumed"),), sender);
}

/// Emitted when the sender cancels the stream.
///
/// `refund_amount` goes back to sender, `accrued_amount` goes to recipient.
pub fn cancelled(env: &Env, sender: &Address, refund_amount: i128, accrued_amount: i128) {
    env.events().publish(
        (symbol_short!("cancelled"), sender),
        (refund_amount, accrued_amount),
    );
}

/// Emitted when the NFT contract updates the stream recipient.
pub fn recipient_updated(env: &Env, old_recipient: &Address, new_recipient: &Address) {
    env.events()
        .publish((symbol_short!("rcpnt_up"),), (old_recipient, new_recipient));
}

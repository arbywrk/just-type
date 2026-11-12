pub mod results;
pub mod typing;

pub enum Screen {
    // Only this state is put inside a pointer because its size makes it resonable.
    Typing(Box<typing::State>),
    Results(results::State),
}

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Joke {
    pub whos_there: &'static str,
    pub answer_who: &'static str,
}

pub const THE_JOKE: Joke = Joke {
    whos_there: "Boo",
    answer_who: "You don't have to cry about it!",
};

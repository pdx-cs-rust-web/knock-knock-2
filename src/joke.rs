#[derive(Clone)]
pub struct Joke {
    pub whos_there: &'static str,
    pub answer: &'static str,
}

pub static THE_JOKE: Joke = Joke {
    whos_there: "Boo",
    answer: "You don't have to cry about it!",
};

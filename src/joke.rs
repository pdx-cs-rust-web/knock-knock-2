pub struct Joke {
    pub whos_there: &'static str,
    pub answer: &'static str,
}

pub const THE_JOKE: Joke = Joke {
    whos_there: "Boo",
    answer: "You don't have to cry about it!",
};

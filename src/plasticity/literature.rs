//! Famous lines of the classics. Name the author (1–2), the work (3–4), or type
//! the author's name (5). Translations are public-domain ones.
use super::{choices, normalize, rng::Rng, Answer, Exercise};

/// (line, author, work).
pub const LINES: [(&str, &str, &str); 20] = [
    ("Call me Ishmael.", "Herman Melville", "Moby-Dick"),
    ("It was the best of times, it was the worst of times…", "Charles Dickens", "A Tale of Two Cities"),
    ("All happy families are alike; each unhappy family is unhappy in its own way.", "Leo Tolstoy", "Anna Karenina"),
    ("It is a truth universally acknowledged, that a single man in possession of a good fortune, must be in want of a wife.", "Jane Austen", "Pride and Prejudice"),
    ("To be, or not to be: that is the question.", "William Shakespeare", "Hamlet"),
    ("Nel mezzo del cammin di nostra vita…", "Dante Alighieri", "Inferno"),
    ("Sing, O goddess, the anger of Achilles son of Peleus…", "Homer", "Iliad"),
    ("In a village of La Mancha, the name of which I have no desire to call to mind…", "Miguel de Cervantes", "Don Quixote"),
    ("As armas e os barões assinalados…", "Luís de Camões", "Os Lusíadas"),
    ("Ao vencedor, as batatas!", "Machado de Assis", "Quincas Borba"),
    ("I am a sick man.... I am a spiteful man.", "Fyodor Dostoevsky", "Notes from Underground"),
    ("One morning, when Gregor Samsa woke from troubled dreams, he found himself transformed in his bed into a horrible vermin.", "Franz Kafka", "The Metamorphosis"),
    ("The unexamined life is not worth living.", "Plato", "Apology"),
    ("Arma virumque cano…", "Virgil", "Aeneid"),
    ("Of Man's first disobedience, and the fruit / Of that forbidden tree…", "John Milton", "Paradise Lost"),
    ("Il faut cultiver notre jardin.", "Voltaire", "Candide"),
    ("Minha terra tem palmeiras, / Onde canta o Sabiá…", "Gonçalves Dias", "Canção do Exílio"),
    ("Tudo vale a pena / Se a alma não é pequena.", "Fernando Pessoa", "Mensagem"),
    ("Whether I shall turn out to be the hero of my own life, or whether that station will be held by anybody else, these pages must show.", "Charles Dickens", "David Copperfield"),
    ("Beatus ille qui procul negotiis…", "Horace", "Epodes"),
];

#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn generate(seed: u64, level: u8) -> Exercise {
    let mut r = Rng::new(seed);
    let (line, author, work) = LINES[r.below(LINES.len() as u64) as usize];
    let (prompt, answer, list) = if level <= 4 {
        let by_work = level >= 3;
        let right = if by_work { work } else { author };
        let wrong: Vec<&str> = LINES
            .iter()
            .filter(|l| l.1 != author)
            .map(|l| if by_work { l.2 } else { l.1 })
            .collect();
        let (c, at) = choices(&mut r, right, &wrong);
        let ask = if by_work {
            "Which work opens or turns on this line?"
        } else {
            "Who wrote it?"
        };
        (format!("“{line}”\n{ask}"), Answer::Choice(at), c)
    } else {
        // the full name, or just the last word of it
        let last = author.rsplit(' ').next().unwrap_or(author);
        let forms = vec![author.to_string(), normalize(last)];
        (
            format!("“{line}”\nWho wrote it? (type the name)"),
            Answer::Text(forms),
            vec![],
        )
    };
    Exercise {
        plast: "first-lines",
        seed,
        level,
        prompt,
        choices: list,
        answer,
        explain: format!("{author}, {work}"),
    }
}

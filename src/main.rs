#![allow(unused)]

use std::{
    cell::RefCell,
    fmt::{Debug, Display},
    ops::Add,
};

struct Arena<T> {
    items: RefCell<Vec<T>>,
}

impl<T> Arena<T> {
    fn new(capacity: usize) -> Self {
        Self {
            items: RefCell::new(Vec::with_capacity(capacity)),
        }
    }

    fn alloc(&self, value: T) -> &T {
        let mut items = self.items.borrow_mut();
        if items.len() == items.capacity() {
            panic!("alloc: cannot allocate further");
        }
        let ptr: *const T = items.push_mut(value);
        drop(items);
        unsafe { &*ptr }
    }

    fn alloc_slice(&self, values: &[T]) -> &[T]
    where
        T: Copy,
    {
        let mut items = self.items.borrow_mut();
        let start = items.len();
        if start + values.len() > items.capacity() {
            panic!("alloc_slice: cannot allocate further");
        }
        items.extend_from_slice(values);
        let ptr: *const T = unsafe { items.as_ptr().add(start) };
        drop(items);
        unsafe { std::slice::from_raw_parts(ptr, values.len()) }
    }

    fn bytes_used(&self) -> usize {
        self.items.borrow().len() * std::mem::size_of::<T>()
    }

    fn bytes_reserved(&self) -> usize {
        self.items.borrow().capacity() * std::mem::size_of::<T>()
    }
}

impl<T: Debug> Arena<T> {
    fn print(&self) {
        println!("{:#?}", self.items.borrow());
    }
}

#[derive(Copy, Clone)]
struct Game<'a> {
    left: &'a [&'a Game<'a>],
    right: &'a [&'a Game<'a>],
}

#[derive(Copy, Clone)]
struct G<'a> {
    ctx: &'a Games<'a>,
    game: &'a Game<'a>,
}

impl<'a> G<'a> {
    fn new(ctx: &'a Games<'a>, left: &[G<'a>], right: &[G<'a>]) -> G<'a> {
        let left = left.iter().map(|x| x.game).collect::<Vec<_>>();
        let right = right.iter().map(|x| x.game).collect::<Vec<_>>();
        let game = ctx.game(&left, &right);
        Self { ctx, game }
    }
}

impl Display for G<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.game, f)
    }
}

impl Display for Game<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.left, self.right) {
            ([], []) => return f.write_str("0"),
            _ => {}
        }
        let mut str = String::new();
        str += "{";
        str += &self
            .left
            .iter()
            .map(|x| format!("{}", x))
            .collect::<Vec<_>>()
            .join(", ");
        str += " | ";
        str += &self
            .right
            .iter()
            .map(|x| format!("{}", x))
            .collect::<Vec<_>>()
            .join(", ");
        str += "}";
        f.write_str(&str)
    }
}

impl PartialEq for G<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.game == other.game
    }
}

impl Eq for G<'_> {}

impl PartialEq for Game<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.left == other.left && self.right == other.right
    }
}

impl Eq for Game<'_> {}

impl<'a> Add for G<'a> {
    type Output = G<'a>;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            ctx: self.ctx,
            game: self.ctx.add(self.game, rhs.game),
        }
    }
}

struct Games<'a> {
    games: Arena<Game<'a>>,
    slices: Arena<&'a Game<'a>>,
}

impl<'a> Games<'a> {
    fn new(games_capacity: usize, slices_capacity: usize) -> Self {
        Self {
            games: Arena::new(games_capacity),
            slices: Arena::new(slices_capacity),
        }
    }

    fn game(&'a self, left: &[&'a Game<'a>], right: &[&'a Game<'a>]) -> &'a Game<'a> {
        let left = self.slices.alloc_slice(left);
        let right = self.slices.alloc_slice(right);
        self.games.alloc(Game { left, right })
    }

    fn add(&'a self, lhs: &'a Game<'a>, rhs: &'a Game<'a>) -> &'a Game<'a> {
        let mut left = vec![];
        let mut right = vec![];
        left.extend(lhs.left.iter().map(|x| self.add(x, rhs)));
        left.extend(rhs.left.iter().map(|x| self.add(x, lhs)));
        right.extend(lhs.right.iter().map(|x| self.add(x, rhs)));
        right.extend(rhs.right.iter().map(|x| self.add(x, lhs)));
        self.game(&left, &right)
    }
}

fn main() {
    let ctx = Games::new(100, 100);
    let zero = G::new(&ctx, &[], &[]);
    let star = G::new(&ctx, &[zero], &[zero]);
    let one = G::new(&ctx, &[zero], &[]);
    let negative_one = G::new(&ctx, &[], &[zero]);
    let g = G::new(&ctx, &[zero], &[star]);
    assert!(star.game.left == star.game.right);
    assert!(zero == G::new(&ctx, &[], &[]));
    println!("{} + {} = {}", one, negative_one, one + negative_one);
    assert!(zero == one + negative_one);
    println!("{}", g);
    println!(
        "{:?} / {:?}",
        ctx.games.bytes_used(),
        ctx.games.bytes_reserved()
    );
    println!(
        "{:?} / {:?}",
        ctx.slices.bytes_used(),
        ctx.slices.bytes_reserved()
    );
}

#[test]
fn test_alloc() {
    let ctx = Games::new(1024, 1024);
    let g = ctx.game(&[], &[]);
    let mut games = vec![];
    for _ in 0..1000 {
        games.push(ctx.game(&[], &[g]));
    }
}

#![allow(unused)]

use std::{
    cell::RefCell,
    cmp::Ordering,
    fmt::{Debug, Display},
    ops::{Add, Neg, Sub},
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
        self.partial_cmp(other)
            .is_some_and(|x| x == Ordering::Equal)
    }
}

impl Eq for G<'_> {}

impl PartialOrd for G<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        debug_assert!(std::ptr::eq(self.ctx, other.ctx));
        let le = self <= other;
        let ge = self >= other;
        match (le, ge) {
            (true, true) => Some(Ordering::Equal),
            (true, false) => Some(Ordering::Less),
            (false, true) => Some(Ordering::Greater),
            (false, false) => None,
        }
    }

    fn le(&self, other: &Self) -> bool {
        debug_assert!(std::ptr::eq(self.ctx, other.ctx));
        self.ctx.le(self.game, other.game)
    }

    fn ge(&self, other: &Self) -> bool {
        other.le(self)
    }
}

impl<'a> Add for G<'a> {
    type Output = G<'a>;

    fn add(self, rhs: Self) -> Self::Output {
        debug_assert!(std::ptr::eq(self.ctx, rhs.ctx));
        Self {
            ctx: self.ctx,
            game: self.ctx.add(self.game, rhs.game),
        }
    }
}

impl<'a> Neg for G<'a> {
    type Output = G<'a>;

    fn neg(self) -> Self::Output {
        Self {
            ctx: self.ctx,
            game: self.ctx.neg(self.game),
        }
    }
}

impl<'a> Sub for G<'a> {
    type Output = G<'a>;

    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
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
        let (mut left, mut right) = self.bypass_reversible(left.to_vec(), right.to_vec());
        left.sort_by_key(|&x| x as *const Game);
        right.sort_by_key(|&x| x as *const Game);
        left.dedup_by(|a, b| std::ptr::eq(*a, *b));
        right.dedup_by(|a, b| std::ptr::eq(*a, *b));

        let left = self.slices.alloc_slice(&left);
        let right = self.slices.alloc_slice(&right);
        self.games.alloc(Game { left, right })
    }

    fn bypass_reversible(
        &'a self,
        mut left: Vec<&'a Game<'a>>,
        mut right: Vec<&'a Game<'a>>,
    ) -> (Vec<&'a Game<'a>>, Vec<&'a Game<'a>>) {
        'outer: loop {
            let start_left = left.len();
            let start_right = right.len();
            left = Self::remove_dominated(left.to_vec(), |a, b| self.le(a, b));
            right = Self::remove_dominated(right.to_vec(), |a, b| self.le(b, a));

            let g = Game {
                left: &left.clone(),
                right: &right.clone(),
            };

            for i in 0..g.left.len() {
                let gl = g.left.get(i).unwrap();
                for ii in 0..gl.right.len() {
                    let glr = gl.right.get(ii).unwrap();
                    if self.le(*glr, &g) {
                        left.extend(left.get(i).unwrap().right.get(ii).unwrap().left);
                        left.remove(i);
                        continue 'outer;
                    }
                }
            }

            for i in 0..g.right.len() {
                let gr = g.right.get(i).unwrap();
                for ii in 0..gr.left.len() {
                    let grl = gr.left.get(ii).unwrap();
                    if self.le(&g, *grl) {
                        right.extend(right.get(i).unwrap().left.get(ii).unwrap().right);
                        right.remove(i);
                        continue 'outer;
                    }
                }
            }

            if start_left == left.len() && start_right == right.len() {
                break;
            }
        }
        (left, right)
    }

    fn remove_dominated(
        options: Vec<&'a Game<'a>>,
        dominated: impl Fn(&Game, &Game) -> bool,
    ) -> Vec<&'a Game<'a>> {
        let mut kept = Vec::new();
        for x in options {
            if kept.iter().any(|&y| dominated(x, y)) {
                continue;
            }
            kept.retain(|&y| !dominated(&y, &x));
            kept.push(x);
        }
        kept
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

    fn neg(&'a self, game: &'a Game<'a>) -> &'a Game<'a> {
        let left = game.right.iter().map(|x| self.neg(x)).collect::<Vec<_>>();
        let right = game.left.iter().map(|x| self.neg(x)).collect::<Vec<_>>();
        self.game(&left, &right)
    }

    fn le(&self, lhs: &Game, rhs: &Game) -> bool {
        for g in lhs.left {
            if self.le(rhs, g) {
                return false;
            }
        }
        for g in rhs.right {
            if self.le(g, lhs) {
                return false;
            }
        }
        true
    }
}

fn main() {
    let ctx = Games::new(1000, 1000);
    let zero = G::new(&ctx, &[], &[]);
    let star = G::new(&ctx, &[zero], &[zero]);
    let one = G::new(&ctx, &[zero], &[]);
    let negative_one = G::new(&ctx, &[], &[zero]);
    let g = G::new(&ctx, &[zero], &[star]);
    println!("{}", one + one + one + one + one);
    println!("{}", one + negative_one);
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
    let ctx = Games::new(1024, 2048);
    let g = ctx.game(&[], &[]);
    let mut games = vec![];
    let mut prev = g;
    for _ in 0..1000 {
        let game = ctx.game(&[prev], &[g]);
        games.push(game);
        prev = game;
    }
}

#[test]
fn test_logic() {
    let ctx = Games::new(100, 100);
    let zero = G::new(&ctx, &[], &[]);
    let star = G::new(&ctx, &[zero], &[zero]);
    let one = G::new(&ctx, &[zero], &[]);
    let negative_one = G::new(&ctx, &[], &[zero]);
    let g = G::new(&ctx, &[zero], &[star]);
    assert!(one == -negative_one);
    assert!(-one == negative_one);
    assert!(one != negative_one);
    assert!(zero == G::new(&ctx, &[], &[]));
    assert!(one + negative_one == zero);
    assert!(star + star == zero);
    assert!(star.partial_cmp(&zero) == None);
    assert!(star.partial_cmp(&g) == None);
    assert!(zero < g);
    assert!(one > g);
}

#[test]
fn dedup_same_pointer() {
    let ctx = Games::new(100, 100);
    let zero = G::new(&ctx, &[], &[]);
    let g = G::new(&ctx, &[zero, zero, zero], &[zero, zero]);
    assert_eq!(g.to_string(), "{0 | 0}");
    assert_eq!(g.game.left.len(), 1);
}

#[test]
fn option_order_does_not_matter() {
    let ctx = Games::new(100, 100);
    let zero = G::new(&ctx, &[], &[]);
    let star = G::new(&ctx, &[zero], &[zero]);
    let a = G::new(&ctx, &[zero, star], &[]);
    let b = G::new(&ctx, &[star, zero], &[]);
    assert_eq!(a.to_string(), b.to_string());
}

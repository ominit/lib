#![allow(unused)]

use std::{
    cell::{Cell, RefCell},
    cmp::Ordering,
    collections::HashMap,
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
impl Game<'_> {
    fn is_zero(&self) -> bool {
        self.left.is_empty() && self.right.is_empty()
    }

    fn as_number(&self) -> Option<(i64, u32)> {
        match (self.left, self.right) {
            ([], []) => Some((0, 0)),
            ([l], []) => match l.as_number()? {
                (n, 0) if n >= 0 => Some((n + 1, 0)),
                _ => None,
            },
            ([], [r]) => match r.as_number()? {
                (n, 0) if n <= 0 => Some((n - 1, 0)),
                _ => None,
            },
            ([l], [r]) => {
                let (a, ak) = l.as_number()?;
                let (b, bk) = r.as_number()?;
                let k = ak.max(bk);
                let (a, b) = (a << (k - ak), b << (k - bk));
                (a < b).then(|| reduce(a + b, k + 1))
            }
            _ => None,
        }
    }

    fn as_nimber(&self) -> Option<usize> {
        let n = self.left.len();
        (self.right.len() == n && is_nimber_set(self.left) && is_nimber_set(self.right))
            .then_some(n)
    }
}

fn reduce(mut n: i64, mut k: u32) -> (i64, u32) {
    while k > 0 && n % 2 == 0 {
        n /= 2;
        k -= 1;
    }
    (n, k)
}

fn is_nimber_set(options: &[&Game]) -> bool {
    let mut seen = vec![false; options.len()];
    for g in options {
        match g.as_nimber() {
            Some(m) if m < seen.len() && !seen[m] => seen[m] = true,
            _ => return false,
        }
    }
    true
}

fn write_number(f: &mut std::fmt::Formatter<'_>, (n, k): (i64, u32)) -> std::fmt::Result {
    match k {
        0 => write!(f, "{n}"),
        k => write!(f, "{n}/{}", 1u64 << k),
    }
}

fn write_options(f: &mut std::fmt::Formatter<'_>, options: &[&Game]) -> std::fmt::Result {
    for (i, g) in options.iter().enumerate() {
        if i > 0 {
            f.write_str(", ")?;
        }
        write!(f, "{g}")?;
    }
    Ok(())
}

impl Display for Game<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(x) = self.as_number() {
            return write_number(f, x);
        }
        if let Some(n) = self.as_nimber() {
            return match n {
                1 => f.write_str("*"),
                n => write!(f, "*{n}"),
            };
        }

        let star = |g: &Game| g.as_nimber() == Some(1);
        let zero_and_star = |a: &Game, b: &Game| a.is_zero() && star(b) || star(a) && b.is_zero();
        match (self.left, self.right) {
            ([l], [r]) if l.is_zero() && star(r) => return f.write_str("↑ "),
            ([l], [r]) if star(l) && r.is_zero() => return f.write_str("↓ "),
            ([a, b], [r]) if r.is_zero() && zero_and_star(a, b) => return f.write_str("↑ *"),
            ([l], [a, b]) if l.is_zero() && zero_and_star(a, b) => return f.write_str("↓ *"),
            ([l], [r]) => {
                if let (Some(x), Some(y)) = (l.as_number(), r.as_number())
                    && x == y
                {
                    write_number(f, x)?;
                    return f.write_str("*");
                }
            }
            _ => {}
        }

        f.write_str("{")?;
        write_options(f, self.left)?;
        f.write_str(" | ")?;
        write_options(f, self.right)?;
        f.write_str("}")
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
    le_calls: Cell<usize>,
    add_calls: Cell<usize>,
    game_calls: Cell<usize>,
    neg_calls: Cell<usize>,
    game_table: RefCell<HashMap<(Vec<usize>, Vec<usize>), &'a Game<'a>>>,
    add_table: RefCell<HashMap<(usize, usize), &'a Game<'a>>>,
    le_table: RefCell<HashMap<(usize, usize), bool>>,
    neg_table: RefCell<HashMap<usize, &'a Game<'a>>>,
}

impl<'a> Games<'a> {
    fn new(games_capacity: usize, slices_capacity: usize) -> Self {
        Self {
            games: Arena::new(games_capacity),
            slices: Arena::new(slices_capacity),
            le_calls: Cell::new(0),
            add_calls: Cell::new(0),
            game_calls: Cell::new(0),
            neg_calls: Cell::new(0),
            game_table: RefCell::new(HashMap::new()),
            add_table: RefCell::new(HashMap::new()),
            le_table: RefCell::new(HashMap::new()),
            neg_table: RefCell::new(HashMap::new()),
        }
    }

    fn print_info(&self) {
        println!(
            "games mem: {:?} / {:?}",
            self.games.bytes_used(),
            self.games.bytes_reserved()
        );
        println!(
            "slices mem: {:?} / {:?}",
            self.slices.bytes_used(),
            self.slices.bytes_reserved()
        );
        println!("game table len: {}", self.game_table.borrow().len());
        println!("add table len: {}", self.add_table.borrow().len());
        println!("le table len: {}", self.le_table.borrow().len());
        println!("neg table len: {}", self.neg_table.borrow().len());
        println!("le calls: {}", self.le_calls.get());
        println!("add calls: {}", self.add_calls.get());
        println!("game calls: {}", self.game_calls.get());
        println!("neg calls: {}", self.neg_calls.get());
    }

    fn game(&'a self, left: &[&'a Game<'a>], right: &[&'a Game<'a>]) -> &'a Game<'a> {
        self.game_calls.update(|x| x + 1);
        let (mut left, mut right) = self.bypass_reversible(left.to_vec(), right.to_vec());
        left.sort_by_key(|&x| x as *const Game);
        right.sort_by_key(|&x| x as *const Game);
        left.dedup_by(|a, b| std::ptr::eq(*a, *b));
        right.dedup_by(|a, b| std::ptr::eq(*a, *b));

        let table = self.game_table.borrow();
        let key = (
            left.iter().map(|&x| (x as *const Game).addr()).collect(),
            right.iter().map(|&x| (x as *const Game).addr()).collect(),
        );
        if let Some(value) = table.get(&key) {
            return value;
        }
        drop(table);

        let left = self.slices.alloc_slice(&left);
        let right = self.slices.alloc_slice(&right);
        let game = self.games.alloc(Game { left, right });
        self.game_table.borrow_mut().insert(key, game);
        game
    }

    fn bypass_reversible(
        &'a self,
        mut left: Vec<&'a Game<'a>>,
        mut right: Vec<&'a Game<'a>>,
    ) -> (Vec<&'a Game<'a>>, Vec<&'a Game<'a>>) {
        let ltable = RefCell::new(HashMap::new());
        let rtable = RefCell::new(HashMap::new());
        left = Self::remove_dominated(left, |a, b| self.le(a, b));
        right = Self::remove_dominated(right, |a, b| self.le(b, a));
        'outer: loop {
            let start_left = left.len();
            let start_right = right.len();

            for i in 0..left.len() {
                let gl = left[i];
                for &glr in gl.right {
                    if self.le_options(glr, &left, &right, &ltable, &rtable) {
                        left.extend(glr.left);
                        left.remove(i);
                        continue 'outer;
                    }
                }
            }

            for i in 0..right.len() {
                let gr = right[i];
                for &grl in gr.left {
                    if self.options_le(&left, &right, grl, &ltable, &rtable) {
                        right.extend(grl.right);
                        right.remove(i);
                        continue 'outer;
                    }
                }
            }

            left = Self::remove_dominated(left, |a, b| self.le(a, b));
            right = Self::remove_dominated(right, |a, b| self.le(b, a));

            if start_left == left.len() && start_right == right.len() {
                break;
            }
        }
        (left, right)
    }

    fn remove_dominated(
        options: Vec<&'a Game<'a>>,
        dominated: impl Fn(&'a Game, &'a Game) -> bool,
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
        self.add_calls.update(|x| x + 1);

        let table = self.add_table.borrow();
        let key = {
            let a = (lhs as *const Game).addr();
            let b = (rhs as *const Game).addr();
            if a > b { (a, b) } else { (b, a) }
        };
        if let Some(value) = table.get(&key) {
            return value;
        }
        drop(table);

        let mut left = vec![];
        let mut right = vec![];
        left.extend(lhs.left.iter().map(|x| self.add(x, rhs)));
        left.extend(rhs.left.iter().map(|x| self.add(x, lhs)));
        right.extend(lhs.right.iter().map(|x| self.add(x, rhs)));
        right.extend(rhs.right.iter().map(|x| self.add(x, lhs)));

        let res = self.game(&left, &right);
        self.add_table.borrow_mut().insert(key, res);
        res
    }

    fn neg(&'a self, game: &'a Game<'a>) -> &'a Game<'a> {
        self.neg_calls.update(|x| x + 1);

        let table = self.neg_table.borrow();
        let key = (game as *const Game).addr();
        if let Some(value) = table.get(&key) {
            return value;
        }
        drop(table);

        let left = game.right.iter().map(|x| self.neg(x)).collect::<Vec<_>>();
        let right = game.left.iter().map(|x| self.neg(x)).collect::<Vec<_>>();

        let game = self.game(&left, &right);
        self.neg_table.borrow_mut().insert(key, game);
        game
    }

    fn le(&self, lhs: &'a Game<'a>, rhs: &'a Game<'a>) -> bool {
        self.le_calls.update(|x| x + 1);
        if std::ptr::eq(lhs, rhs) {
            return true;
        }

        let table = self.le_table.borrow();
        let key = ((lhs as *const Game).addr(), (rhs as *const Game).addr());
        if let Some(b) = table.get(&key) {
            return *b;
        }
        drop(table);

        let f = |b: bool| {
            self.le_table.borrow_mut().insert(key, b);
        };

        for g in lhs.left {
            if self.le(rhs, g) {
                f(false);
                return false;
            }
        }
        for g in rhs.right {
            if self.le(g, lhs) {
                f(false);
                return false;
            }
        }
        f(true);
        true
    }

    fn options_le(
        &self,
        left: &[&'a Game<'a>],
        right: &[&'a Game<'a>],
        rhs: &'a Game<'a>,
        ltable: &RefCell<HashMap<usize, bool>>,
        rtable: &RefCell<HashMap<usize, bool>>,
    ) -> bool {
        self.le_calls.update(|x| x + 1);

        let key = (rhs as *const Game).addr();
        if let Some(b) = ltable.borrow().get(&key) {
            return *b;
        }

        let f = |b: bool| {
            ltable.borrow_mut().insert(key, b);
        };

        for &g in left {
            if self.le(rhs, g) {
                f(false);
                return false;
            }
        }
        for &g in rhs.right {
            if self.le_options(g, left, right, ltable, rtable) {
                f(false);
                return false;
            }
        }
        f(true);
        true
    }

    fn le_options(
        &self,
        lhs: &'a Game<'a>,
        left: &[&'a Game<'a>],
        right: &[&'a Game<'a>],
        ltable: &RefCell<HashMap<usize, bool>>,
        rtable: &RefCell<HashMap<usize, bool>>,
    ) -> bool {
        self.le_calls.update(|x| x + 1);

        let key = (lhs as *const Game).addr();
        if let Some(b) = rtable.borrow().get(&key) {
            return *b;
        }

        let f = |b: bool| {
            rtable.borrow_mut().insert(key, b);
        };

        for &g in lhs.left {
            if self.options_le(left, right, g, ltable, rtable) {
                f(false);
                return false;
            }
        }
        for &g in right {
            if self.le(g, lhs) {
                f(false);
                return false;
            }
        }
        f(true);
        true
    }
}

fn main() {
    let start = std::time::Instant::now();
    let ctx = Games::new(1 << 20, 1 << 20);
    let zero = G::new(&ctx, &[], &[]);
    let star = G::new(&ctx, &[zero], &[zero]);
    let one = G::new(&ctx, &[zero], &[]);
    let negative_one = G::new(&ctx, &[], &[zero]);
    let g = G::new(&ctx, &[zero], &[star]);
    let up = G::new(&ctx, &[zero], &[star]);

    let a = g + g + g + g + g + g + g;
    let b = one + negative_one - a;
    let c = a + b - a + star - g;
    let d = a + b + c + b;
    let mut nimbers = vec![zero];
    for _ in 1..=100 {
        let next = G::new(&ctx, &nimbers, &nimbers);
        nimbers.push(next);
    }

    for n in [10, 15, 20, 40, 100] {
        let sum = (1..n).fold(up, |acc, _| acc + up);
    }

    for n in [4, 5, 6, 10, 30, 50] {
        let sum = nimbers[2..=n].iter().fold(nimbers[1], |acc, &x| acc + x);
    }
    println!("total time: {:?}", start.elapsed());
    ctx.print_info();
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
    assert_eq!(g.to_string(), "*");
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

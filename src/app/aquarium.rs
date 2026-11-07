//! Frutiger Aero aquarium: the fish in the sky as a small simulation. They wander and school with
//! their own kind, chase and gulp the bubbles, puff out bubbles of their own, dart away from the
//! pointer and turn back at the edges. Runs only while the Aero wallpaper is painted.

use super::bubbles::{self, Prey};
use super::fishes::{self, FishPose};
use super::glass;
use super::ornaments::noise;
use super::pixel::mix;
use super::style::Palette;
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
use std::f32::consts::TAU;

const FISH_COUNT: u32 = 12;
/// Longest simulated step, so a stalled or hidden window doesn't fling the fish away.
const MAX_STEP: f32 = 0.1;
const TOP_SPEED: f32 = 260.0;
/// How far a hungry fish notices a bubble worth chasing.
const HUNT_REACH: f32 = 260.0;
const FLEE_REACH: f32 = 150.0;
const FLEE_SPEED: f32 = 240.0;
/// Fish keep this far inside the window's top and bottom, as fractions of its height.
const SWIM_BAND: (f32, f32) = (0.18, 0.86);
const EDGE_MARGIN: f32 = 70.0;
const GULP_SECONDS: f32 = 0.35;
const BREATH_SECONDS: f32 = 2.6;
const BREATH_LIMIT: usize = 80;

#[derive(Clone)]
struct Fish {
    position: Pos2,
    velocity: Vec2,
    /// Smoothly follows the swimming direction, -1 for left and 1 for right; a fish turning
    /// around passes through 0 and briefly shows its narrow front.
    facing: f32,
    seed: u32,
    length: f32,
    /// Time of the last meal, for the gulp and the pause before the next hunt.
    fed: f32,
    next_breath: f32,
    /// Phase of the tail beat, which quickens with speed.
    stroke: f32,
    /// Fright from 0 to 1: rises fast near the pointer and settles slowly.
    alarm: f32,
}

/// A little bubble a fish breathed out.
#[derive(Clone)]
struct Breath {
    origin: Pos2,
    born: f32,
    seed: u32,
}

#[derive(Clone, Default)]
struct Aquarium {
    fish: Vec<Fish>,
    breaths: Vec<Breath>,
    last_time: Option<f32>,
    released: u32,
}

/// What the fish can see this frame.
struct Surroundings<'a> {
    screen: Rect,
    prey: &'a [Prey],
    pointer: Option<Pos2>,
    time: f32,
}

/// A fish's body pushing small bubbles aside as it swims through them.
pub struct Wake {
    pub center: Pos2,
    pub reach: f32,
}

fn state_id() -> egui::Id {
    egui::Id::new("aero_aquarium")
}

/// Moves the fish one frame, lets them eat and breathe, and paints them.
pub fn swim(
    ctx: &egui::Context,
    painter: &egui::Painter,
    screen: Rect,
    palette: &Palette,
    night: bool,
) {
    let time = ctx.input(|input| input.time) as f32;
    let mut aquarium: Aquarium = ctx.data_mut(|data| data.get_temp(state_id()).unwrap_or_default());
    if aquarium.fish.is_empty() {
        aquarium.fish = (0..FISH_COUNT)
            .map(|seed| hatch(seed, screen, time))
            .collect();
    }
    let step = aquarium
        .last_time
        .map_or(0.0, |last| (time - last).clamp(0.0, MAX_STEP));
    aquarium.last_time = Some(time);
    let prey = bubbles::prey(ctx, palette);
    let surroundings = Surroundings {
        screen,
        prey: &prey,
        pointer: ctx.input(|input| input.pointer.hover_pos()),
        time,
    };
    for eaten in simulate(&mut aquarium, &surroundings, step) {
        bubbles::burst(ctx, eaten, time);
    }
    for breath in &aquarium.breaths {
        paint_breath(painter, breath, time, palette);
    }
    for fish in &aquarium.fish {
        paint(painter, fish, time, night);
    }
    ctx.data_mut(|data| data.insert_temp(state_id(), aquarium));
}

/// Where the fish are, for the bubbles to make way.
pub fn wakes(ctx: &egui::Context) -> Vec<Wake> {
    let aquarium: Option<Aquarium> = ctx.data_mut(|data| data.get_temp(state_id()));
    aquarium.map_or_else(Vec::new, |aquarium| {
        aquarium
            .fish
            .iter()
            .map(|fish| Wake {
                center: fish.position,
                reach: fish.length * 0.8,
            })
            .collect()
    })
}

fn hatch(seed: u32, screen: Rect, time: f32) -> Fish {
    let random = |salt: u32| noise(seed, salt + 900);
    let (low, high) = SWIM_BAND;
    let direction = if random(0) > 0.5 { 1.0 } else { -1.0 };
    Fish {
        position: egui::pos2(
            screen.left() + random(1) * screen.width(),
            screen.top() + screen.height() * (low + random(2) * (high - low)),
        ),
        velocity: Vec2::new(direction * 30.0, 0.0),
        facing: direction,
        seed,
        length: 32.0 + random(3) * 34.0,
        fed: time - 10.0,
        next_breath: time + random(4) * 4.0,
        stroke: random(5) * TAU,
        alarm: 0.0,
    }
}

/// Advances every fish by `step` seconds; returns the bubbles they ate.
fn simulate(aquarium: &mut Aquarium, world: &Surroundings, step: f32) -> Vec<(u32, u32)> {
    let others: Vec<(Pos2, Vec2, u32)> = aquarium
        .fish
        .iter()
        .map(|fish| (fish.position, fish.velocity, fishes::school(fish.seed)))
        .collect();
    let mut eaten = Vec::new();
    let mut puffs = Vec::new();
    for fish in &mut aquarium.fish {
        let target = fish
            .hungry(world.time)
            .then(|| fish.nearest_prey(world.prey, &eaten))
            .flatten();
        let (desired, agility) = fish.desire(world, &others, target);
        fish.velocity += (desired - fish.velocity) * (agility * step).min(1.0);
        let speed = fish.velocity.length();
        if speed > TOP_SPEED {
            fish.velocity *= TOP_SPEED / speed;
        }
        fish.position += fish.velocity * step;
        if fish.velocity.x.abs() > 4.0 {
            let turn = fish.velocity.x.signum() - fish.facing;
            fish.facing += turn * (step * 4.0).min(1.0);
        }
        fish.stroke += step * (5.0 + fish.velocity.length() * 0.08);
        let scared = fish.escape(world.pointer).is_some();
        let calm = if scared { 1.0 } else { 0.0 };
        let pace = if scared { 6.0 } else { 0.6 };
        fish.alarm += (calm - fish.alarm) * (step * pace).min(1.0);
        if let Some(prey) = target.filter(|prey| fish.mouth().distance(prey.center) < prey.radius) {
            eaten.push(prey.key);
            fish.fed = world.time;
            puffs.push((fish.mouth(), 3 + fish.seed % 3));
        }
        if world.time >= fish.next_breath {
            fish.next_breath = world.time + 3.0 + noise(fish.seed, aquarium.released) * 5.0;
            puffs.push((fish.mouth(), 1 + aquarium.released % 3));
        }
    }
    for (mouth, count) in puffs {
        breathe(aquarium, mouth, count, world.time);
    }
    aquarium
        .breaths
        .retain(|breath| world.time - breath.born < BREATH_SECONDS);
    eaten
}

impl Fish {
    fn random(&self, salt: u32) -> f32 {
        noise(self.seed, salt + 950)
    }

    fn cruise(&self) -> f32 {
        28.0 + self.random(0) * 30.0
    }

    fn hungry(&self, time: f32) -> bool {
        time - self.fed > 2.0 + self.random(1) * 5.0
    }

    fn pitch(&self) -> f32 {
        self.velocity
            .y
            .atan2(self.velocity.x.abs().max(1.0))
            .clamp(-0.6, 0.6)
    }

    fn mouth(&self) -> Pos2 {
        let reach = self.length * 0.5 * self.facing.signum();
        self.position + Vec2::angled(self.pitch() * self.facing.signum()) * reach
    }

    /// The closest bubble in reach that no other fish has just eaten.
    fn nearest_prey<'a>(&self, prey: &'a [Prey], eaten: &[(u32, u32)]) -> Option<&'a Prey> {
        prey.iter()
            .filter(|prey| !eaten.contains(&prey.key))
            .map(|prey| (prey, prey.center.distance(self.position)))
            .filter(|(_, distance)| *distance < HUNT_REACH)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(prey, _)| prey)
    }

    /// The velocity this fish wants now and how quickly it can change towards it.
    fn desire(
        &self,
        world: &Surroundings,
        others: &[(Pos2, Vec2, u32)],
        target: Option<&Prey>,
    ) -> (Vec2, f32) {
        if let Some(away) = self.escape(world.pointer) {
            let dash = away + Vec2::new(0.0, self.random(2) - 0.5);
            return (dash.normalized() * FLEE_SPEED, 6.0);
        }
        let mut desired = match target {
            Some(prey) => (prey.center - self.mouth()).normalized() * self.cruise() * 2.4,
            None => self.wander(world.time),
        };
        desired += self.school(others) + self.keep_inside(world.screen);
        (desired, if target.is_some() { 2.5 } else { 1.2 })
    }

    /// The direction away from the pointer while it is close enough to scare this fish.
    fn escape(&self, pointer: Option<Pos2>) -> Option<Vec2> {
        let away = self.position - pointer?;
        (away.length() < FLEE_REACH).then(|| away.normalized())
    }

    /// A slowly changing heading, so fish meander and now and then turn around.
    fn wander(&self, time: f32) -> Vec2 {
        let angle = time * (0.04 + self.random(3) * 0.06)
            + self.random(4) * TAU
            + (time * 0.23 + self.random(5) * TAU).sin() * 0.9;
        Vec2::new(angle.cos(), 0.45 * (angle * 1.7).sin()) * self.cruise()
    }

    /// Room from every fish close by, and a pull towards the pace of its own kind.
    fn school(&self, others: &[(Pos2, Vec2, u32)]) -> Vec2 {
        let mut push = Vec2::ZERO;
        let mut pace = Vec2::ZERO;
        let mut mates = 0.0;
        for &(position, velocity, kind) in others {
            let offset = self.position - position;
            let distance = offset.length();
            if distance < 0.01 {
                continue;
            }
            let room = self.length * 1.6;
            if distance < room {
                push += offset.normalized() * (room - distance) * 2.0;
            }
            if kind == fishes::school(self.seed) && distance < 220.0 {
                pace += velocity;
                mates += 1.0;
            }
        }
        if mates > 0.0 {
            push += (pace / mates - self.velocity) * 0.3;
        }
        push
    }

    /// A push back towards the swimming band once a fish strays near the window's edges.
    fn keep_inside(&self, screen: Rect) -> Vec2 {
        let (low, high) = SWIM_BAND;
        let top = screen.top() + screen.height() * low;
        let bottom = screen.top() + screen.height() * high;
        let along = |value: f32, from: f32, to: f32| {
            if value < from + EDGE_MARGIN {
                (from + EDGE_MARGIN - value) * 1.5
            } else if value > to - EDGE_MARGIN {
                (to - EDGE_MARGIN - value) * 1.5
            } else {
                0.0
            }
        };
        Vec2::new(
            along(self.position.x, screen.left(), screen.right()),
            along(self.position.y, top, bottom),
        )
    }
}

fn breathe(aquarium: &mut Aquarium, mouth: Pos2, count: u32, time: f32) {
    for _ in 0..count {
        if aquarium.breaths.len() >= BREATH_LIMIT {
            return;
        }
        let seed = aquarium.released;
        aquarium.released = aquarium.released.wrapping_add(1);
        aquarium.breaths.push(Breath {
            origin: mouth,
            born: time + noise(seed, 970) * 0.25,
            seed,
        });
    }
}

fn paint(painter: &egui::Painter, fish: &Fish, time: f32, night: bool) {
    let gulp = 1.0 - ((time - fish.fed) / GULP_SECONDS).clamp(0.0, 1.0);
    let length = fish.length * (1.0 + 0.15 * gulp);
    let pose = FishPose {
        center: fish.position,
        length,
        heading: fish.facing,
        pitch: fish.pitch(),
        wag: fish.stroke.sin(),
        seed: fish.seed,
        alarm: fish.alarm,
    };
    fishes::paint_fish(painter, &pose, night);
}

/// A tiny bubble rising from a fish's mouth, wobbling and swelling as it goes.
fn paint_breath(painter: &egui::Painter, breath: &Breath, time: f32, palette: &Palette) {
    let age = time - breath.born;
    if age < 0.0 {
        return;
    }
    let random = |salt: u32| noise(breath.seed, salt + 980);
    let t = age / BREATH_SECONDS;
    let fade = ((1.0 - t) * 3.0).min(1.0) * (age * 10.0).min(1.0);
    let rise = (10.0 + random(0) * 10.0) * age + 0.5 * (20.0 + random(1) * 25.0) * age * age;
    let wobble = (age * (4.0 + random(2) * 4.0) + random(3) * TAU).sin() * (1.0 + 3.0 * t);
    let center = breath.origin + Vec2::new(wobble, -rise);
    let radius = (1.5 + random(4) * 3.0) * (1.0 + t * 0.4);
    let rim = mix(palette.hot, Color32::WHITE, 0.4).gamma_multiply(0.85 * fade);
    painter.circle_filled(center, radius, palette.hot.gamma_multiply(0.12 * fade));
    painter.circle_stroke(center, radius, Stroke::new(1.0_f32, rim));
    painter.circle_filled(
        center + Vec2::splat(-radius * 0.35),
        (radius * 0.3).max(0.6),
        Color32::WHITE.gamma_multiply(0.9 * fade),
    );
    if radius > 3.5 {
        painter.add(glass::radial_glow(
            center,
            radius * 1.6,
            palette.hot.gamma_multiply(0.1 * fade),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn screen() -> Rect {
        Rect::from_min_size(Pos2::ZERO, egui::vec2(1000.0, 800.0))
    }

    fn run(aquarium: &mut Aquarium, prey: &[Prey], pointer: Option<Pos2>, seconds: f32) {
        let mut time = 0.0;
        while time < seconds {
            time += 1.0 / 30.0;
            let world = Surroundings {
                screen: screen(),
                prey,
                pointer,
                time,
            };
            simulate(aquarium, &world, 1.0 / 30.0);
        }
    }

    #[test]
    fn fish_stay_near_the_window_while_wandering_for_minutes() {
        let mut aquarium = Aquarium {
            fish: (0..FISH_COUNT)
                .map(|seed| hatch(seed, screen(), 0.0))
                .collect(),
            ..Default::default()
        };
        run(&mut aquarium, &[], None, 300.0);
        let bounds = screen().expand(EDGE_MARGIN * 2.0);
        for fish in &aquarium.fish {
            assert!(bounds.contains(fish.position), "{:?}", fish.position);
            assert!(fish.velocity.length() <= TOP_SPEED + 1e-3);
        }
        assert!(aquarium.breaths.len() <= BREATH_LIMIT);
        assert!(!aquarium.breaths.is_empty(), "fish breathe out bubbles");
    }

    #[test]
    fn a_hungry_fish_catches_a_bubble_ahead_of_it() {
        let mut fish = hatch(0, screen(), -20.0);
        fish.position = egui::pos2(400.0, 400.0);
        let prey = [Prey {
            key: (7, 1),
            center: egui::pos2(520.0, 430.0),
            radius: 14.0,
        }];
        let mut aquarium = Aquarium {
            fish: vec![fish],
            ..Default::default()
        };
        let mut time = 0.0;
        let caught = (0..300).any(|_| {
            time += 1.0 / 30.0;
            let world = Surroundings {
                screen: screen(),
                prey: &prey,
                pointer: None,
                time,
            };
            simulate(&mut aquarium, &world, 1.0 / 30.0) == vec![(7, 1)]
        });
        assert!(caught);
    }

    #[test]
    fn a_scared_fish_puffs_up_and_calms_down_again() {
        let mut fish = hatch(6, screen(), 0.0);
        fish.position = egui::pos2(500.0, 400.0);
        let mut aquarium = Aquarium {
            fish: vec![fish],
            ..Default::default()
        };
        run(&mut aquarium, &[], Some(egui::pos2(480.0, 400.0)), 0.4);
        assert!(aquarium.fish[0].alarm > 0.8);
        run(&mut aquarium, &[], None, 10.0);
        assert!(aquarium.fish[0].alarm < 0.1);
    }

    #[test]
    fn fish_dart_away_from_the_pointer() {
        let mut fish = hatch(1, screen(), 0.0);
        fish.position = egui::pos2(500.0, 400.0);
        let pointer = egui::pos2(460.0, 400.0);
        let mut aquarium = Aquarium {
            fish: vec![fish],
            ..Default::default()
        };
        run(&mut aquarium, &[], Some(pointer), 0.5);
        assert!(aquarium.fish[0].position.distance(pointer) > 70.0);
    }
}

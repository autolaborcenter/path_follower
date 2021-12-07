﻿use crate::{isometry, vector};
use nalgebra::{Complex, Isometry2, Vector2};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI, SQRT_2};

/// 计算面积比并转化到 [-π/2, π/2]
pub fn track(part: &[Isometry2<f32>], light_radius: f32) -> Option<f32> {
    let squared = light_radius.powi(2);
    let delta = vector(light_radius, 0.0);

    // 查找路段起点、终点
    let mut begin = part[0];
    begin.translation.vector -= delta;
    if begin.translation.vector.norm_squared() > squared {
        return None;
    }
    let end = part
        .iter()
        .skip(1)
        .map(|p| {
            let mut p = *p;
            p.translation.vector -= delta;
            p
        })
        .take_while(|p| p.translation.vector.norm_squared() < squared)
        .last()
        .unwrap_or(part[0]);

    let begin = intersection(&begin, squared, -1.0);
    let end = intersection(&end, squared, 1.0);
    let diff = angle_of(end) - angle_of(begin); // [-2π, 2π]
    Some((diff.signum() * PI - diff) / 2.0) // [-π/2, π/2]
}

pub fn goto(target: Isometry2<f32>, light_radius: f32) -> (f32, f32) {
    const FRAC_PI_16: f32 = PI / 16.0;

    // 退出临界角
    // 目标方向小于此角度时考虑退出
    let theta = FRAC_PI_16; // assert θ < π/4

    // 原地转安全半径
    // 目标距离小于此半径且目标方向小于临界角时可退出
    let squared = {
        let rho = SQRT_2 * light_radius;
        let theta = 3.0 * FRAC_PI_4 + theta; // 3π/4 + θ
        let (sin, cos) = theta.sin_cos();
        let vec = vector(light_radius + rho * cos, rho * sin);
        vec.norm_squared() * 0.95 // 略微收缩确保可靠性
    };

    // 光斑中心相对机器人的位姿
    let c_light = isometry(light_radius, 0.0, 1.0, 0.0);
    // 机器人坐标系上机器人应该到达的目标位置
    let target = target * c_light;

    let p = target.translation.vector;
    let d = target.rotation.angle();

    let l = p.norm_squared();
    // 位置条件满足
    if l < squared {
        return if d.abs() < theta {
            // 位置方向条件都满足，退出
            (0.0, 0.0)
        } else {
            // 方向条件不满足，原地转
            (1.0, d.signum() * -FRAC_PI_2)
        };
    }
    // 位置条件不满足，逼近
    let speed = f32::min(1.0, l.sqrt() * 0.5);
    let dir = -p[1].atan2(p[0]);
    // 后方不远
    return if p[0] > -1.0 && dir.abs() > FRAC_PI_4 * 3.0 {
        (p[0].signum() * speed, dir.signum() * PI - dir)
    } else {
        (speed, dir.clamp(-FRAC_PI_2, FRAC_PI_2))
    };
}

/// 求射线与圆交点
fn intersection(p: &Isometry2<f32>, r_squared: f32, signnum: f32) -> Vector2<f32> {
    let vp = p.translation.vector;
    let vd = dir_vector(p);

    // let a = 1.0;
    let b = 2.0 * vp.dot(&vd);
    let c = vp.norm_squared() - r_squared;

    #[allow(non_snake_case)]
    let Δ = b.powi(2) - 4.0 * c;
    let k = (-b + signnum * Δ.sqrt()) / 2.0;
    vp + vd * k
}

/// 求方向向量
#[inline]
fn dir_vector(p: &Isometry2<f32>) -> Vector2<f32> {
    let Complex { re, im } = *p.rotation.complex();
    vector(re, im)
}

/// 求方向角
#[inline]
fn angle_of(p: Vector2<f32>) -> f32 {
    p.y.atan2(p.x)
}

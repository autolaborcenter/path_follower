﻿use crate::{isometry, point, vector, Isometry2, Vector2};
use nalgebra::Complex;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_3, FRAC_PI_4, PI};

/// 计算面积比并转化到 [-π/2, π/2]
pub(crate) fn track(
    slice: &[Isometry2<f32>],
    pose: Isometry2<f32>,
    light_radius: f32,
) -> Option<(usize, f32)> {
    let c = (pose * point(light_radius, 0.0)).coords;
    let squared = light_radius.powi(2);
    let i = slice
        .iter()
        .enumerate()
        .take(20)
        .find(|(_, p)| (c - p.translation.vector).norm_squared() < squared)
        .map(|(i, _)| i)?;

    // 查找路段起点、终点
    let local = (pose * isometry(light_radius, 0.0, 1.0, 0.0)).inverse();
    let begin = local * slice[i];
    let end = slice
        .iter()
        .skip(i)
        .take_while(|p| (c - p.translation.vector).norm_squared() < squared)
        .last()
        .map_or(begin, |p| local * p);

    let begin = intersection(&begin, squared, -1.0);
    let end = intersection(&end, squared, 1.0);
    let diff = angle_of(end) - angle_of(begin);
    Some((i, (diff.signum() * PI - diff) / 2.0)) // [-π/2, π/2]
}

/// 上目标点
pub(crate) fn goto(target: Isometry2<f32>, light_radius: f32) -> Option<(f32, f32)> {
    // 机器人坐标系上机器人应该到达的目标位置
    let target = target * isometry(-light_radius, 0.0, 1.0, 0.0);

    let p = target.translation.vector;
    let d = target.rotation.angle();

    let l = p.norm_squared();
    // 位置条件满足
    if l < light_radius.powi(2) {
        // 转到面朝线
        if (p[1].is_sign_positive() && d.is_sign_negative() && -FRAC_PI_3 < d)
            || (p[1].is_sign_negative() && d.is_sign_positive() && d < FRAC_PI_3)
        {
            // 位置方向条件都满足，退出
            None
        } else {
            // 方向条件不满足，原地转
            Some((1.0, d.signum() * -FRAC_PI_2))
        }
    } else {
        // 位置条件不满足，逼近
        let mut speed = f32::min(1.0, (2.0 - d.abs() / PI) * l.sqrt());
        let mut dir = -p[1].atan2(p[0]);
        // 后方不远
        if p[0] > -light_radius * 0.5 && dir.abs() > FRAC_PI_4 * 3.0 {
            speed *= p[0].signum();
            dir = dir.signum() * PI - dir
        }
        Some((
            speed,
            dir.clamp(-FRAC_PI_2, FRAC_PI_2) / f32::max(1.0, p[0]),
        ))
    }
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

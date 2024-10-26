/*
 * Copyright 2024 Akash Rawal
 *
 * This file is part of Disposables.
 *
 * Disposables is free software: you can redistribute it and/or modify it under 
 * the terms of the GNU General Public License as published by the 
 * Free Software Foundation, either version 3 of the License, or 
 * (at your option) any later version.
 * 
 * Disposables is distributed in the hope that it will be useful, 
 * but WITHOUT ANY WARRANTY; without even the implied warranty of 
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. 
 * See the GNU General Public License for more details.
 * 
 * You should have received a copy of the GNU General Public License 
 * along with Disposables. If not, see <https://www.gnu.org/licenses/>. 
 */
//!Test utility functions

/**
 * Runs a given closure on each element in an iterator, 
 * and returns the first Ok result or all errors.
 */
pub fn try_use<I, F, R, E>(iter: I, mut f: F) -> Result<R, Vec<E>>
where I: IntoIterator,
      F: FnMut(<I as IntoIterator>::Item) -> Result<R, E>
{
    let mut errors = Vec::new();

    for i in iter {
        match f(i) {
            Ok(r) => return Ok(r),
            Err(e) => errors.push(e),
        }
    }
    Err(errors)
}



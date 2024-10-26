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
//!Test utility functions (async)

use std::future::Future;

use futures::stream::FuturesUnordered;
use futures::StreamExt;

/**
 * Runs a given closure on each element in an iterator, 
 * and returns the first Ok result or all errors.
 *
 * This function runs the closure sequentially on each element from the beginning.
 * Only after the closure finishes for the first element, will it run for
 * the next element.
 */
pub async fn try_use_seq<I, F, R, E, Fut>(iter: I, mut f: F) -> Result<R, Vec<E>>
where I: IntoIterator,
      F: FnMut(<I as IntoIterator>::Item) -> Fut,
      Fut: Future<Output = Result<R, E>>
{
    let mut errors = Vec::new();

    for i in iter {
        match f(i).await {
            Ok(r) => return Ok(r),
            Err(e) => errors.push(e),
        }
    }
    Err(errors)
}

/**
 * Runs a given closure on each element in an iterator, 
 * and returns the first Ok result or all errors.
 *
 * This function runs the closure concurrently on all elements.
 * Once one closure invocation finishes with Ok result, it is returned
 * and all other futures are dropped.
 */
pub async fn try_use<I, F, R, E, Fut>(iter: I, mut f: F) -> Result<R, Vec<E>>
where I: IntoIterator,
      F: FnMut(<I as IntoIterator>::Item) -> Fut,
      Fut: Future<Output = Result<R, E>>
{
    let mut futures = FuturesUnordered::new();

    for (idx, i) in (0..).zip(iter) {
        let fut = f(i);
        futures.push(async move {
            match fut.await {
                Ok(r) => Ok(r),
                Err(e) => Err((idx, e)),
            }
        });
    }

    let mut errors = std::iter::repeat_with(|| None)
        .take(futures.len()).collect::<Vec<Option<E>>>();

    while let Some(el) = futures.next().await {
        match el {
            Ok(r) => return Ok(r),
            Err((idx, e)) => errors[idx] = Some(e),
        }
    }

    Err(errors.into_iter().map(Option::unwrap).collect())
}

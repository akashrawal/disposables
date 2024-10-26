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
//! Convenient data type to represent command-line arguments.

/**
 * A struct representing command-line argument list.
 */
#[derive(Default)]
pub struct Args {
    args: Vec<String>,
}


impl Args {
    /**
     * Creates a new, empty list.
     */
    pub fn new() -> Self {
        Default::default()
    }

    /**
     * Adds a single argument to the list.
     */
    pub fn add(&mut self, arg: impl AsRef<str>) -> &mut Self {
        self.args.push(arg.as_ref().to_owned());
        self
    }

    /**
     * Adds arguments to the list from the given iterator.
     */
    pub fn extend<T>(&mut self, args: T) -> &mut Self
        where T: IntoIterator,
              <T as IntoIterator>::Item: AsRef<str>
    {
        args.into_iter().for_each(|s| { self.add(s); });
        self
    }

    /**
     * Gets a reference to the argument list.
     */
    pub fn get(&self)-> &[String] {
        &self.args
    }

    /**
     * Consumes the argument list and returns it as a vector.
     */
    pub fn into_vec(self) -> Vec<String> {
        self.args
    }
}

impl<T> From<T> for Args
where T: IntoIterator,
      <T as IntoIterator>::Item: AsRef<str>
{
    fn from(value: T) -> Self {
        Self {
            args: value.into_iter()
                .map(|s| s.as_ref().to_owned())
                .collect()
        }
    }
}



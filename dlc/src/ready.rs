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
//Ready/timeout state tracker

use std::cell::RefCell;

use disposables_protocol::V1Event;
use tokio::sync::mpsc::Sender;

pub struct ReadySignal {
    value: RefCell<i32>,
    sender: Sender<V1Event>,
}

impl ReadySignal {
    pub fn new(value: i32, sender:Sender<V1Event>) -> Self {
        Self {
            value: RefCell::new(value),
            sender
        }
    }
    pub async fn dec(&self, by: i32) {
        if by > 0 {
            let value = {
                let mut value = self.value.borrow_mut();
                *value -= by;
                *value
            };
            if value == 0 {
                self.sender.send(V1Event::Ready).await
                    .expect("Cannot send event");
            }
        }
    }
    pub async fn timeout(&self) {
        let prev_value = {
            let mut value = self.value.borrow_mut();
            let prev_value = *value;
            *value = 0;
            prev_value
        };
        if prev_value > 0 {
            self.sender.send(V1Event::FailedTimeout).await
                .expect("Cannot send event");
        }
    }
}

#[cfg(test)]
mod test {
    use tokio::sync::mpsc::channel;

    use super::*;

    #[tokio::test]
    async fn when_wait_for_list_is_empty_then_no_signal_is_sent_on_timeout() {
        let (sender, mut receiver) = channel(1);
        let s = ReadySignal::new(0, sender);
        s.timeout().await;
        drop(s);
        assert!(receiver.recv().await.is_none());
    }

    #[tokio::test]
    async fn when_wait_for_list_is_not_empty_then_timeout_can_be_sent() {
        let (sender, mut receiver) = channel(1);
        let s = ReadySignal::new(1, sender);
        s.timeout().await;
        drop(s);
        assert!(matches!(receiver.recv().await, Some(V1Event::FailedTimeout)));
        assert!(receiver.recv().await.is_none());
    }

    #[tokio::test]
    async fn when_wait_for_list_is_finished_ready_signal_is_sent() {
        let (sender, mut receiver) = channel(1);
        let s = ReadySignal::new(1, sender);
        s.dec(1).await;
        drop(s);
        assert!(matches!(receiver.recv().await, Some(V1Event::Ready)));
        assert!(receiver.recv().await.is_none());
    }

    #[tokio::test]
    async fn after_ready_signal_timeout_cannot_be_sent() {
        let (sender, mut receiver) = channel(1);
        let s = ReadySignal::new(1, sender);
        s.dec(1).await;
        s.timeout().await;
        drop(s);
        assert!(matches!(receiver.recv().await, Some(V1Event::Ready)));
        assert!(receiver.recv().await.is_none());
    }

    #[tokio::test]
    async fn afteer_timeout_seady_signal_cannot_be_sent() {
        let (sender, mut receiver) = channel(1);
        let s = ReadySignal::new(1, sender);
        s.timeout().await;
        s.dec(1).await;
        drop(s);
        assert!(matches!(receiver.recv().await, Some(V1Event::FailedTimeout)));
        assert!(receiver.recv().await.is_none());
    }
}

/*
 * Copyright (c) 2018 Boucher, Antoni <bouanto@zoho.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

use std::io::{BufRead, BufReader, Read};
use std::num::ParseIntError;
use std::time::Duration;

use Month::*;

#[derive(Debug, PartialEq)]
pub enum Month {
    January = 0,
    February = 1,
    March = 2,
    April = 3,
    May = 4,
    June = 5,
    July = 6,
    August = 7,
    September = 8,
    October = 9,
    November = 10,
    December = 11,
}

#[derive(Debug, PartialEq)]
pub struct Date {
    pub day: u8,
    pub month: Month,
    pub year: u16,
}

#[derive(Debug, PartialEq)]
pub struct Time {
    pub hour: u8,
    pub minute: u8,
}

#[derive(Debug, PartialEq)]
pub struct Entry {
    pub date: Date,
    pub duration: Duration,
    pub msg: String,
    pub time: Time,
}

pub fn parse<R: Read>(reader: R) -> Result<Vec<Entry>, String> {
    let mut entries = vec![];
    let reader = BufReader::new(reader);
    for line in reader.lines() {
        let line = line.map_err(|error| error.to_string())?;
        let mut parser = Parser::new(&line);
        if let Ok(entry) = parser.entry() {
            entries.push(entry);
        }
    }
    Ok(entries)
}

struct Parser {
    index: usize,
    words: Vec<String>,
}

impl Parser {
    fn new(line: &str) -> Self {
        let words = line.split_whitespace()
            .filter(|word| !word.trim().is_empty())
            .map(ToString::to_string)
            .collect();
        Self {
            index: 0,
            words,
        }
    }

    fn date(&mut self) -> Result<Date, String> {
        let month =
            match self.next_word().ok_or_else(|| "Expecting date, found end of line".to_string())?.to_lowercase().as_str() {
                "jan" => January,
                "feb" => February,
                "mar" => March,
                "apr" => April,
                "may" => May,
                "jun" => June,
                "jul" => July,
                "aug" => August,
                "sep" => September,
                "oct" => October,
                "nov" => November,
                "dec" => December,
                month => return Err(format!("Invalid month {}", month)),
            };
        let day = self.num()? as u8;
        let year = self.num()? as u16;
        Ok(Date {
            day,
            month,
            year,
        })
    }

    fn duration(&mut self) -> Result<Duration, String> {
        self.ident("DURATION")?;
        let time = self.time_num()?;
        Ok(Duration::from_secs(time.hour as u64 * 60 * 60 + time.minute as u64 * 60))
    }

    fn entry(&mut self) -> Result<Entry, String> {
        self.ident("REM")?;
        let date = self.date()?;
        let time = self.time()?;
        let duration = self.duration()?;
        let msg = self.message()?;
        Ok(Entry {
            date,
            duration,
            msg,
            time,
        })
    }

    fn ident(&mut self, ident: &str) -> Result<(), String> {
        if self.next_word().map(str::to_lowercase) != Some(ident.to_lowercase()) {
            return Err("Expecting REM at beginning of line".to_string());
        }
        Ok(())
    }

    fn message(&mut self) -> Result<String, String> {
        self.ident("MSG")?;
        let message = self.words[self.index..].join(" ");
        Ok(message)
    }

    fn next_word(&mut self) -> Option<&str> {
        let index = self.index;
        let result = self.words.get(index)
            .map(|string| string.as_str());
        if result.is_some() {
            self.index += 1;
        }
        result
    }

    fn num(&mut self) -> Result<u32, String> {
        self.next_word()
            .ok_or_else(|| "Expecting day of month, found end of line".to_string())?
            .parse()
            .map_err(|error: ParseIntError| error.to_string())
    }

    fn time(&mut self) -> Result<Time, String> {
        self.ident("AT")?;
        let time = self.time_num()?;
        Ok(time)
    }

    fn time_num(&mut self) -> Result<Time, String> {
        let time = self.next_word().ok_or_else(|| "Expecting time, found end of line".to_string())?;
        let mut parts = time.split(':');
        let hour = parts.next()
            .ok_or_else(|| "Expecting hour, found end of line".to_string())
            .map_err(|error| error.to_string())?
            .parse()
            .map_err(|error: ParseIntError| error.to_string())?;
        let minute = parts.next()
            .ok_or_else(|| "Expecting hour, found end of line".to_string())
            .map_err(|error| error.to_string())?
            .parse()
            .map_err(|error: ParseIntError| error.to_string())?;
        Ok(Time {
            hour,
            minute,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use {Date, Time, parse};
    use Month::*;

    #[test]
    fn parse_rem() {
        let data = "REM Mar 30 2018 AT 19:00 DURATION 1:15 MSG Event name";
        let entries = parse(data.as_bytes()).expect("entries");
        assert_eq!(entries[0].date, Date { day: 30, month: March, year: 2018 });
        assert_eq!(entries[0].duration, Duration::from_secs(75 * 60));
        assert_eq!(entries[0].msg, "Event name".to_string());
        assert_eq!(entries[0].time, Time { hour: 19, minute: 0 });

        let data = "REM Mar 30 2018 AT 19:00 DURATION 1:15 MSG Event name
        REM Apr 9 2018 AT 12:50 DURATION 0:15 MSG Super Event";
        let entries = parse(data.as_bytes()).expect("entries");
        assert_eq!(entries[0].date, Date { day: 30, month: March, year: 2018 });
        assert_eq!(entries[0].duration, Duration::from_secs(75 * 60));
        assert_eq!(entries[0].msg, "Event name".to_string());
        assert_eq!(entries[0].time, Time { hour: 19, minute: 0 });
        assert_eq!(entries[1].date, Date { day: 9, month: April, year: 2018 });
        assert_eq!(entries[1].duration, Duration::from_secs(15 * 60));
        assert_eq!(entries[1].msg, "Super Event".to_string());
        assert_eq!(entries[1].time, Time { hour: 12, minute: 50 });
    }
}

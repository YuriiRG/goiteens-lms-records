# LMS recods script

A CLI program to upload lesson records to GoITeens LMS. It can upload and remove the records. Written in [Rust](https://www.rust-lang.org/).

## Features

- Obviously, uploading lesson records.
- Reading records from a tab-separated file. It's the format you get when copying from Google Sheets.
- Automatically filtering out lessons without links.
- Support for lessons with multiple space-separated links/records.
- Deduplication of lessons with the same name.
- Automatic record type deduction ("video" or "other").
- Truncation of lesson names longer than the allowed limit (70 characters).
- Prepending lesson names with their type (Tech skills/Soft skills).
- Not prepending lessons with their type if it's already in the name.
- Descriptive errors.
- Removing records. Useful in case you make a mistake.
- Easy login using command-line arguments or secure login using environment variables. `.env` is supported too.
- Comprehensive documentation in CLI help.
- Optional per-record progress report. Can be turned off using quiet mode.

## Installation

Download the latest binary from the [Releases page](https://github.com/YuriiRG/goiteens-lms-records/releases), unless you want to install [Rust toolchain](https://www.rust-lang.org/tools/install) and compile it from source.

## Usage

There is a comperhensive documention in the CLI help. For example, it can be invoked like this:

```powershell
.\lms-records.exe help
.\lms-records.exe --help
.\lms-records.exe help upload
.\lms-records.exe upload --help
```

It should answer all you questions about how to use the program. However, in general, first you need to log in your LMS admin account:

```powershell
.\lms-records.exe login <USERNAME> <PASSWORD>
```

Alternatively, you can put your username and password into environment variables LMS_USERNAME and LMS_PASSWORD respectively and use `login-env` command. `.env` file is supported.

```powershell
.\lms-records.exe login-env
```

Afterwards, you will probably want to use the `upload` command. Before you do it, you should put the records you want to upload in the `input.txt` file in the working directory (in the directory you call the program from). `input.txt` format is described [below](#inputtxt-format). Also, you need to find the id of the group you want to upload the records for. It can be found in the group's URL. In the URL of the page where you would upload records manually it's the first number.

```powershell
.\lms-records.exe upload <GROUP_ID>
```

## `input.txt` format

Each line is a lesson in a tab-separated format, like what you get when copying from Google Sheets. Tech skills and soft skills are separated with double newline. Tech skills come first, soft skills come second.

Example:

```
Tech skills lesson 1 12.06<TAB>https://youtu.be/ExAmPlE
Lesson 2 19.06<TAB>https://drive.google.com/ExAmPlE
Tech skills lesson 42 08.01<TAB>https://youtu.be/ExAmPlE2 https://youtu.be/ExAmPlE3

Soft_skills_lesson_1_12.06<TAB>
Lesson 2 19.06<TAB>https://drive.google.com/ExAmPlE2
```

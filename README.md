# chimecho
Hi there :D! As a musician and ML Engineer, I have been interested in using AI to help augment my music production process. One aspect of my music production process is sound design around percussion and using audio signal processing techniques to get the specific drum sound that I want. However, this process can be a little tedious, and oftentimes I end up making sounds with similar timbranal qualities, transients, etc. I began to think: What if I could build a model that could generate novel percussion sounds for me? In order to start the modeling process, I need a system to pull down some music samples. Enter chimecho! 

Chimecho is a CLI program written in Rust that pulls music samples that were posted on the [Drumkits](https://www.reddit.com/r/Drumkits/) subreddit. This works by pulling down all of the posts based on a specified query via the [PushShift API](https://github.com/pushshift/api); routing traffic to the link within the post to Dropbox, Google Drive, and Mediafire; and then downloading the files from these storage options. From here, the program will then uncompress the files, store the metadata in a Postgres DB, and then uploads the uncompressed data to a Google Cloud Storage (GCS) bucket.

## How to get started
In order to run chimecho, the following needs to be in place:
1. You will need to have Rust and cargo installed on your system (TODO: will be creating a binary for all platforms).
2. You will need to have `7z`, `rar`, `gsutil`, `docker-compose` installed on your system. 
3. You will need to set an environment variable for `GOOGLE_APPLICATION_CREDENTIALS` in your `.bashrc`, or `.zshrc` in order to access the Google Drive API and the Google Cloud Bucket you would like to use. 
4. You will need to set a `DATABASE_URL` that will be used to connect to the Postgres DB for the metadata store. 

## How to run the program
First, clone this repo. Then navigate to the `docker` subfolder and run `docker-compose up` in a separate terminal. This will start the Postgres DB instance. 

Note: **Please update the password, username, and name of the DB to reflect your `DATABASE_URL` within the `docker/docker-compose.yml` file**. They are currently set to test as placeholders. 

Once the DB is up and running, execute the `create table` statements in the `sql/create_tables.sql` script. 
Once that is done, you may go ahead and run the program.
There are 2 subcommands for chimecho: `download` and `upload`.
### Download
The download subcommand is used to get the compressed music files and stores it locally on your machine.
```
USAGE:
    chimecho download [OPTIONS] --file-path <FILE_PATH>

OPTIONS:
    -f, --file-path <FILE_PATH>        File path folder for the music data to live in
    -h, --help                         Print help information
    -q, --q <Q>                        Optional query string for Reddit API. Can get more info here:
                                       https://github.com/pushshift/api
    -s, --step-size <STEP_SIZE>        Number of steps to iterate over posts list
    -t, --time-period <TIME_PERIOD>    Optional time period. Specified using UTC or day format.
                                       Example: --time-period "after=7d" Example:
                                       "after=1586604030&before=1605097230"USAGE:
    chimecho download [OPTIONS] --file-path <FILE_PATH>
```

Example: 
```
cargo run -- download --file-path data/ --time-period "after=1586604030&before=1605097230"
```

### Upload
The upload subcommand is used to upload the uncompressed music files, stores the metadata for each file, and uploads to GCS
```
USAGE:
    chimecho upload --file-path <FILE_PATH> --bucket <BUCKET>

OPTIONS:
    -b, --bucket <BUCKET>          bucket name for google cloud storage upload
    -f, --file-path <FILE_PATH>    File path folder that contains zip and rar files
    -h, --help                     Print help information
```

Example:
```
cargo run -- upload --file-path data/ --bucket chimecho_bucket
```
### Misc
In order to run tests please use this command:
`cargo test`

Formatting the code on the project can be executed via:
`cargo fmt`

For linting purposes I make use of `clippy`. The installation instructions can be found [here](https://github.com/rust-lang/rust-clippy):
`cargo clippy`

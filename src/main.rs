use reqwest::Client;
use async_recursion::async_recursion;
use reqwest::Error;
use rust_embed::RustEmbed;
use std::{io::Write, collections::HashMap};
use clap::Parser;
use serde::{Deserialize,Serialize};
use indicatif::ProgressBar;
use std::fs;

#[derive(RustEmbed)]
#[folder = "queries"]
#[prefix = "queries/"]
struct Asset;

#[derive(Parser, Debug)]
#[clap(name = "Get Music", author, version, about, long_about=None)]

pub struct Args {
    // Artist
    #[arg(short, long)]
    pub artist: Option<String>,
}

#[derive(Deserialize, Debug)]
struct SongsResponse {
    data: SongData
}

#[derive(Deserialize, Debug)]
struct ArtistResponse {
    data: ArtistData 
}

#[derive(Deserialize, Debug)]
struct ArtistData {
    user: User
}

#[derive(Deserialize, Debug)]
struct SongData {
    node: User
}

#[derive(Deserialize, Debug)]
struct User {
    id: String,
    uploads: Uploads
}

#[derive(Deserialize, Debug)]
struct Uploads {
    edges: Vec<Edge>,
    pageInfo: PageInfo
}

#[derive(Deserialize, Debug)]
struct PageInfo {
    endCursor: String,
    hasNextPage: bool
}

#[derive(Deserialize, Debug)]
struct Edge {
    node: Node,
    cursor: String
}

#[derive(Deserialize, Debug)]
struct Node {
    id: String,
    name: String,
    previewUrl: String
}

/*struct Cache {*/
    /*file: String*/
/*}*/

/*impl Cache {*/
    /*fn read(self) -> Result<HashMap<String, Vec<&'static str>>, Box<dyn std::error::Error>> {*/
        /*let file_contents = fs::read_to_string("address.txt")?;*/
        /*let cache: HashMap<String, Vec<&str>> = HashMap::new();*/
        /*let artists: Vec<&str> = file_contents.split("\n").collect();*/
        /*artists.into_iter().for_each(|artist| {*/
            /*let (a, tracks_str) = artist.rsplit_once(":").unwrap();*/
            /*let tracks: Vec<&str> = tracks_str.split(";").collect();*/
            /*cache.insert(a.to_string(), tracks);*/
        /*});*/
        /*Ok(cache)*/
    /*}*/

    /*fn save(self, content: String) -> Result<(), &'static str> {*/
        /*match fs::write(self.file, content) {*/
            /*Ok(_) => Ok(()),*/
            /*Err(_) => Err("Error writting in file")*/
        /*}*/
    /*}*/
/*}*/

impl Node {
    async fn download(self) -> Result<(), Error> {
        let file_name = format!("{}.m4a", self.name);
        let (_, path) = self.previewUrl.rsplit_once("previews/").unwrap();
        let song_uuid = path.replace(".mp3", ".m4a");
        let mp3_url = format!("https://stream1.mixcloud.com/secure/c/m4a/64/{}?sig=aktXwbujA7gIzbLBIYTlYQ", song_uuid);
        let mut response = reqwest::get(&mp3_url).await.unwrap();
        match response.status() {
            reqwest::StatusCode::OK => {
                println!("Downloading {} from {}", self.name, mp3_url);
                let size = response.content_length().unwrap();
                let progress = ProgressBar::new(size);
                let mut file = fs::File::create(&file_name).unwrap();
                while let Some(chunk) = response.chunk().await? {
                    progress.inc(chunk.len() as u64);
                    let _ = file.write_all(&chunk);
                }
            }
            _other => {
                println!("url {} failed", &mp3_url);
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Debug)]
struct Payload<'a> {
    query: &'a str,
    variables: Variables<'a>
}

#[derive(Serialize, Debug)]
struct Variables<'a> {
    lookup: Option<Lookup<'a>>,
    orderBy: String,
    count: u32,
    cursor: Option<String>,
    id: Option<String>
}

#[derive(Serialize, Debug)]
struct Lookup<'a> {
    username: &'a String
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();
    let data = get_data(args.artist.unwrap()).await.unwrap();
    println!("Found {} tracks.", data.len());
    for node in data {
        let _ = node.download().await;
    }
    Ok(())
}

#[async_recursion]
async fn recursive_pages(
    get_songs: String,
    client: Client,
    id: String,
    cursor: Option<String>,
    page: u8
) -> Result<Vec<Node>, String> {
    let variables = Variables {
        lookup: None,
        orderBy: "LATEST".to_string(),
        count: 50,
        cursor,
        id: Some(id.to_string())
    };
    let payload = Payload {
        query: &get_songs,
        variables
    };
    let response = client.post("https://app.mixcloud.com/graphql")
        .header("Host", "app.mixcloud.com")
        .json(&payload)
        .send()
        .await
        .unwrap();
    println!("Getting page {}: {}\n{}", page, response.status(), serde_json::json!(&payload).to_string());
    match response.status() {
        reqwest::StatusCode::OK => {
            let text = response.text().await.unwrap();
            println!("{}", text); 
            let res = serde_json::from_str::<SongsResponse>(&text);
            match res {
                Ok(res) => {
                    println!("Response\n{:?}", &res.data);
                    let mut uploads: Vec<Node> = res.data.node.uploads.edges.into_iter().map(|x| x.node).collect();
                    println!("Found {} results.", uploads.len());
                    if res.data.node.uploads.pageInfo.hasNextPage {
                        match recursive_pages(
                            get_songs,
                            client,
                            id.to_owned(),
                            Some(res.data.node.uploads.pageInfo.endCursor),
                            page+1
                        ).await {
                            Ok(mut results) => uploads.append(&mut results),
                            Err(_) => {}
                        };
                    }
                    Ok(uploads)
                },
                Err(error) => {
                    Err(format!("{}", error))
                }
            }
            
        },
        status => Err(format!("Error fetching request\n{}", status))
    }
}

async fn get_data(artist: String) -> Result<Vec<Node>, &'static str> {
    let client = Client::builder()
        .cookie_store(true)
        .user_agent("PostmanRuntime/7.32.3")
        .build()
        .unwrap();

    let get_artist = Asset::get("queries/getArtist.gql").unwrap();

    let lookup = Lookup { username: &artist };
    let variables = Variables {
        lookup: Some(lookup),
        orderBy: "LATEST".to_string(),
        count: 1000,
        cursor: None,
        id: None
    };
    let payload = Payload {
        query: std::str::from_utf8(&get_artist.data.as_ref()).unwrap(),
        variables
    };
    let response = client.post("https://app.mixcloud.com/graphql")
        .header("Host", "app.mixcloud.com")
        .json(&payload)
        .send()
        .await
        .unwrap();
    
    match response.status() {
        reqwest::StatusCode::OK => {
            let get_songs = Asset::get("queries/getSongs.gql").unwrap();
            let res = response.json::<ArtistResponse>().await.unwrap();
            let mut uploads: Vec<Node> = res.data.user.uploads.edges.into_iter().map(|x| x.node).collect();
            println!("{}", uploads.len());
            if res.data.user.uploads.pageInfo.hasNextPage {
                let mut results = recursive_pages(
                    std::str::from_utf8(&get_songs.data.as_ref()).unwrap().to_string().replace("\\n", ""),
                    client,
                    res.data.user.id,
                    Some(res.data.user.uploads.pageInfo.endCursor),
                    0
                ).await.unwrap();
                uploads.append(&mut results);
            }
            Ok(uploads)
        },
        _other => Err("test")
    }
}

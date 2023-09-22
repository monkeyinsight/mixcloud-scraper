use reqwest::Client;
use async_recursion::async_recursion;
use reqwest::ClientBuilder;
use std::future::Future;
use std::pin::Pin;
use reqwest::Error;
use regex::Regex;
use rust_embed::RustEmbed;
use std::collections::HashMap;
use std::io::{copy,Cursor};
use clap::Parser;
use serde::{Deserialize,Serialize};

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
    slug: String,
    name: String,
    audioType: String,
    previewUrl: String
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
    count: u8,
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
    let data = get_data("otographic".to_string()/* args.artist.unwrap() */).await;
    println!("{:?}", data.unwrap());
    Ok(())
}

#[async_recursion]
async fn recursive_pages(
    get_songs: String,
    client: Client,
    id: String,
    cursor: Option<String>
) -> Result<Vec<Node>, String> {
    let variables = Variables {
        lookup: None,
        orderBy: "LATEST".to_string(),
        count: 20,
        cursor,
        id: Some(id.to_string())
    };
    let payload = Payload {
        query: &get_songs,
        variables
    };
    println!("{:?}", serde_json::json!(&payload).to_string());
    let response = client.post("https://app.mixcloud.com/graphql")
        .header("Host", "app.mixcloud.com")
        .json(&payload)
        .send()
        .await
        .unwrap();

    match response.status() {
        reqwest::StatusCode::OK => {
            let res = response.json::<SongsResponse>().await.unwrap();
            println!("{:?}", res);
            let mut uploads: Vec<Node> = res.data.node.uploads.edges.into_iter().map(|x| x.node).collect();
            if res.data.node.uploads.pageInfo.hasNextPage {
                let mut results = recursive_pages(get_songs, client, id.to_owned(), Some(res.data.node.uploads.pageInfo.endCursor)).await.unwrap();
                uploads.append(&mut results);
            }
            Ok(uploads)
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
        count: 20,
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
            if res.data.user.uploads.pageInfo.hasNextPage {
                let mut results = recursive_pages(std::str::from_utf8(&get_songs.data.as_ref()).unwrap().to_string(), client, res.data.user.id, Some(res.data.user.uploads.pageInfo.endCursor)).await.unwrap();
                uploads.append(&mut results);
            }
            Ok(uploads)
        },
        _other => Err("test")
    }
    
    // let response = client.get(format!("https://www.mixcloud.com/{}/", &artist)).send().await.unwrap();
    // match response.status() {
    //     reqwest::StatusCode::OK => {
    //         let body = response.text().await?;
    //         let re = Regex::new(r#"\"name\":\"([^"]+?)\",\"audi.+?\"previewUrl\":\"(.+?)\""#).unwrap();
    //         let mut results: Vec<Track> = vec![];

    //         for (_, [name, url]) in re.captures_iter(&body).map(|c| c.extract()) {
    //             results.push(Track { name: name.to_owned(), url: url.to_owned() });
    //         }
    //         println!("{:?}", results);
    //         for track in results {
    //             let _ = fetch_mixcloud(track).await;
    //         }
    //     }
    //     _other => println!("Unexpected status code: {}", response.status())
    // }
}

// async fn fetch_mixcloud(track: Track) -> Result<(), Error> {
//     let file_name = format!("{}.m4a", track.name);
//     let (_, path) = track.url.rsplit_once("previews/").unwrap();
//     let song_uuid = path.replace(".mp3", ".m4a");
//     for server in 1..23 {
//         let mp3_url = format!("https://stream{}.mixcloud.com/secure/c/m4a/64/{}?sig=aktXwbujA7gIzbLBIYTlYQ", server.to_string(), song_uuid);
//         let response = reqwest::get(&mp3_url).await;
//         match response {
//             Ok(response) => {
//                 match response.status() {
//                     reqwest::StatusCode::OK => {
//                         let mut file = std::fs::File::create(&file_name).unwrap();
//                         let mut content = Cursor::new(response.bytes().await.unwrap());
//                         copy(&mut content, &mut file).unwrap();
//                     }
//                     _other => {
//                         println!("url {} failed", &mp3_url);
//                     }
//                 }
//             }
//             Error => {}
//         }
//     }
//     Ok(())
// }

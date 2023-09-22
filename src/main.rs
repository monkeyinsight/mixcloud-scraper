use reqwest::Client;
use reqwest::Error;
use regex::Regex;
use std::collections::HashMap;
use std::io::{copy,Cursor};
use clap::Parser;
use serde::{Deserialize,Serialize};

#[derive(Parser, Debug)]
#[clap(name = "Get Music", author, version, about, long_about=None)]

pub struct Args {
    // Artist
    #[arg(short, long)]
    pub artist: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Response {
    data: Data 
}

#[derive(Deserialize, Debug)]
struct Data {
    user: User
}

#[derive(Deserialize, Debug)]
struct User {
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
    query: String,
    variables: Variables<'a>
}

#[derive(Serialize, Debug)]
struct Variables<'a> {
    lookup: Lookup<'a>,
    orderBy: String
}

#[derive(Serialize, Debug)]
struct Lookup<'a> {
    username: &'a String
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();
    let _ = get_mixcloud("otographic".to_string()/* args.artist.unwrap() */).await;
    Ok(())
}

#[derive(Debug)]
pub struct Track {
    name: String,
    url: String
}

async fn get_mixcloud(artist: String) -> Result<(), Error> {
    let client = Client::builder()
        .cookie_store(true)
        .user_agent("PostmanRuntime/7.32.3")
        .build()?;

    let query = r#"query UserUploadsQuery(
    $lookup: UserLookup!
    $orderBy: CloudcastOrderByEnum
) {
    user: userLookup(lookup: $lookup) {
        username
        ...UserUploadsPage_user_7FfCv
        id
    }
    viewer {
        ...UserUploadsPage_viewer
        id
    }
}

fragment AudioCardActions_cloudcast on Cloudcast {
    id
    isPublic
    slug
    isExclusive
    isUnlisted
    isScheduled
    isDraft
    audioType
    isDisabledCopyright
    owner {
        id
        username
        isSubscribedTo
        isViewer
        affiliateUsers {
            totalCount
        }
    }
    ...AudioCardFavoriteButton_cloudcast
    ...AudioCardRepostButton_cloudcast
    ...AudioCardShareButton_cloudcast
    ...AudioCardAddToButton_cloudcast
    ...AudioCardHighlightButton_cloudcast
    ...AudioCardBoostButton_cloudcast
    ...AudioCardStats_cloudcast
}

fragment AudioCardActions_viewer on Viewer {
    me {
        uploadLimits {
            tracksPublishRemaining
                showsPublishRemaining
        }
        id
    }
    ...AudioCardFavoriteButton_viewer
    ...AudioCardRepostButton_viewer
    ...AudioCardHighlightButton_viewer
}

fragment AudioCardAddToButton_cloudcast on Cloudcast {
    id
    isUnlisted
    isPublic
}

fragment AudioCardBoostButton_cloudcast on Cloudcast {
    id
    isPublic
    owner {
        id
        isViewer
    }
}

fragment AudioCardFavoriteButton_cloudcast on Cloudcast {
    id
    isFavorited
    isPublic
    hiddenStats
    favorites {
        totalCount
    }
    slug
    owner {
        id
        isFollowing
        username
        isSelect
        displayName
        isViewer
    }
}

fragment AudioCardFavoriteButton_viewer on Viewer {
    me {
        id
    }
}

fragment AudioCardHighlightButton_cloudcast on Cloudcast {
    id
    isPublic
    isHighlighted
    owner {
        isViewer
        id
    }
}

fragment AudioCardHighlightButton_viewer on Viewer {
    me {
        id
        hasProFeatures
        highlighted {
            totalCount
        }
    }
}

fragment AudioCardPlayButton_cloudcast on Cloudcast {
    id
    restrictedReason
    owner {
        displayName
        country
        username
        isSubscribedTo
        isViewer
        id
    }
    slug
    isAwaitingAudio
    isDraft
    isPlayable
    streamInfo {
        hlsUrl
        dashUrl
        url
        uuid
    }
    audioLength
    currentPosition
    proportionListened
    repeatPlayAmount
    hasPlayCompleted
    seekRestriction
    previewUrl
    isExclusivePreviewOnly
    isExclusive
    isDisabledCopyright
}

fragment AudioCardProgress_cloudcast on Cloudcast {
    id
    proportionListened
    audioLength
}

fragment AudioCardRepostButton_cloudcast on Cloudcast {
    id
    isReposted
    isExclusive
    isPublic
    reposts {
        totalCount
    }
    owner {
        isViewer
        isSubscribedTo
        id
    }
}

fragment AudioCardRepostButton_viewer on Viewer {
    me {
        id
    }
}

fragment AudioCardShareButton_cloudcast on Cloudcast {
    id
    isUnlisted
    isPublic
    slug
    description
    audioType
    picture {
        urlRoot
    }
    owner {
        displayName
        isViewer
        username
        id
    }
}

fragment AudioCardStats_cloudcast on Cloudcast {
    isExclusive
    isDraft
    hiddenStats
    plays
    publishDate
    qualityScore
    listenerMinutes
    owner {
        isSubscribedTo
        id
    }
    tags(country: \"GLOBAL\") {
        tag {
            name
            slug
            id
        }
    }
    ...AudioCardTags_cloudcast
}

fragment AudioCardSubLinks_cloudcast on Cloudcast {
    id
    isExclusive
    owner {
        id
        displayName
        username
        ...Hovercard_user
    }
    creatorAttributions(first: 2) {
        totalCount
        edges {
            node {
                id
                displayName
                username
                ...Hovercard_user
            }
        }
    }
}

fragment AudioCardTags_cloudcast on Cloudcast {
    tags(country: \"GLOBAL\") {
        tag {
            name
            slug
            id
        }
    }
}

fragment AudioCardTitle_cloudcast on Cloudcast {
    id
    slug
    name
    audioType
    audioQuality
    isLiveRecording
    isExclusive
    owner {
        id
        username
        ...UserBadge_user
    }
    creatorAttributions(first: 2) {
        totalCount
    }
    ...AudioCardSubLinks_cloudcast
    ...AudioCardPlayButton_cloudcast
    ...ExclusiveCloudcastBadgeContainer_cloudcast
}

fragment AudioCard_cloudcast on Cloudcast {
    id
    slug
    name
    audioType
    isAwaitingAudio
    isDraft
    isScheduled
    restrictedReason
    publishDate
    isLiveRecording
    isDisabledCopyright
    owner {
        isViewer
        username
        id
    }
    picture {
        ...UGCImage_picture
    }
    ...AudioCardTitle_cloudcast
    ...AudioCardProgress_cloudcast
    ...AudioCardActions_cloudcast
    ...QuantcastCloudcastTracking_cloudcast
}

fragment AudioCard_viewer on Viewer {
    ...AudioCardActions_viewer
    me {
        uploadLimits {
            tracksPublishRemaining
            showsPublishRemaining
        }
        id
    }
}

fragment ExclusiveCloudcastBadgeContainer_cloudcast on Cloudcast {
    isExclusive
    isExclusivePreviewOnly
    slug
    id
    owner {
        username
        id
    }
}

fragment Hovercard_user on User {
    id
}

fragment QuantcastCloudcastTracking_cloudcast on Cloudcast {
    owner {
        quantcastTrackingPixel
        id
    }
}

fragment ShareAudioCardList_user on User {
    biog
    username
    displayName
    id
    isUploader
    picture {
        urlRoot
    }
}

fragment UGCImage_picture on Picture {
    urlRoot
    primaryColor
}

fragment UserBadge_user on User {
    hasProFeatures
    isStaff
    hasPremiumFeatures
}

fragment UserUploadsPage_user_7FfCv on User {
    id
    displayName
    username
    isViewer
    ...ShareAudioCardList_user
    uploads(first: 10, isPublic: true, orderBy: $orderBy, audioTypes: [SHOW]) {
        edges {
            node {
                ...AudioCard_cloudcast
                id
                __typename
            }
            cursor
        }
        pageInfo {
            endCursor
            hasNextPage
        }
    }
}

fragment UserUploadsPage_viewer on Viewer {
    ...AudioCard_viewer
}"#;
        let lookup = Lookup { username: &artist };
        let variables = Variables {
            lookup,
            orderBy: "LATEST".to_string()
        };
        let payload = Payload {
            query: query.to_string(),
            variables
        };
        let response = client.post("https://app.mixcloud.com/graphql")
            .header("Host", "app.mixcloud.com")
            .json(&payload)
            .send()
            .await?;

        match response.status() {
            reqwest::StatusCode::OK => {
                println!("{:?}", response.json::<Response>().await?);
            }
            _other => println!("Unexpected status code: {}", response.status())
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

        Ok(())
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

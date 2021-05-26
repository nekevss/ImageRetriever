use bytes::Bytes;
use regex::Regex;
use reqwest;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::result::Result;

use crate::RetrieverState;

pub enum ImageType {
    Jpeg,
    Png,
    Tiff,
    Other,
}

//need to keep reference to RetrieveState all on one page...at least that's my current approach
pub fn request_runner(data: &mut RetrieverState) {
    //So we first need to parse our requests
    let request_input = data.requests_input.clone();
    if request_input != "" {
        let parsed_requests = parse_requests(request_input);

        for request in parsed_requests.iter() {
            log_feedback(data, format!("Running request for {}", request));
            run_request(data, request);
        }
    } else {
        log_feedback(data, String::from("There was no input provided to fetch"));
    }
}

fn parse_requests(requests_string: String) -> Vec<String> {
    let newline_regex = Regex::new("\r*\n").unwrap();
    let mut request_vector = Vec::new();

    //check if there is a new line
    if newline_regex.is_match(&requests_string) {
        //let requests_iter: Vec<&str> = newline_regex.split(&requests_string).collect();
        //requests_iter.iter()
        for request in newline_regex
            .split(&requests_string)
            .filter(|x| !x.is_empty())
        {
            request_vector.push(String::from(request.clone()));
        }
    } else {
        request_vector.push(requests_string);
    };

    request_vector
}

///function wrapper for running the request
fn run_request(data: &mut RetrieverState, request_url: &String) {
    let request_response = reqwest::blocking::get(request_url.clone());

    match request_response {
        Ok(response) => {
            log_feedback(data, String::from("Response returned successfully!"));
            log_feedback(data, String::from("Checking response content type..."));
            let headers = response.headers();
            let content_type = headers.get("content-type");
            if let Some(content) = content_type {
                let image_type = is_image_type(content.clone());
                match image_type {
                    ImageType::Other => log_feedback(
                        data,
                        String::from(
                            "Invalid or Unsupported HTTP Content Type: Could not process request",
                        ),
                    ),
                    _ => {
                        log_feedback(data, String::from("Content type is supported"));
                        let download_result =
                            run_image_download(data, image_type, response, request_url);
                        match download_result {
                            Ok(_x) => log_feedback(data, String::from("Download Complete")),
                            Err(err) => log_feedback(
                                data,
                                format!("Download aborted! An error occurred: {}", err),
                            ),
                        }
                    }
                }
            } else {
                log_feedback(
                    data,
                    String::from(
                        "Error: response header contained no content type. Could not download.",
                    ),
                )
            };
        }
        Err(err) => {
            //This should be any error result from the term.
            //So we just have to log the error in our feedback.
            log_feedback(
                data,
                format!("Http request failed! Returned an error: {}", err),
            )
        }
    }
}

fn is_image_type(content_type: reqwest::header::HeaderValue) -> ImageType {
    if content_type == "image/jpeg" {
        return ImageType::Jpeg;
    } else if content_type == "image/png" {
        return ImageType::Png;
    } else if content_type == "image/tiff" {
        return ImageType::Tiff;
    } else {
        return ImageType::Other;
    };
}

fn run_image_download(
    data: &mut RetrieverState,
    content_type: ImageType,
    response: reqwest::blocking::Response,
    request: &String,
) -> std::io::Result<()> {
    log_feedback(data, String::from("Beginning download..."));
    let response_bytes = response.bytes();
    match response_bytes {
        Ok(bytes) => {
            match content_type {
                //match the content types
                ImageType::Jpeg => {
                    //special handling cause jpg spelling is blah
                    if request.contains(".JPG") {
                        let new_request = request.replace(".JPG", ".jpg");
                        let file_name = determine_file_name(&new_request, "jpg");
                        handle_image_bytes(data, bytes, &file_name)
                    } else if request.contains(".jpeg") {
                        let new_request = request.replace(".jpeg", ".jpg");
                        let file_name = determine_file_name(&new_request, ".jpg");
                        handle_image_bytes(data, bytes, &file_name)
                    } else {
                        let file_name = determine_file_name(request, ".jpg");
                        handle_image_bytes(data, bytes, &file_name)
                    }
                }
                ImageType::Png => {
                    let file_name = determine_file_name(request, ".png");
                    handle_image_bytes(data, bytes, &file_name)
                }
                ImageType::Tiff => {
                    let file_name = determine_file_name(request, ".tiff");
                    handle_image_bytes(data, bytes, &file_name)
                }
                _ => Ok(()),
            }
        }
        Err(err) => {
            log_feedback(
                data,
                format!("Error while reading bytes from response body: {}", err),
            );
            Ok(())
        }
    }
}

fn determine_file_name(request_string: &String, suffix: &str) -> String {
    if request_string.contains(suffix) {
        let request_path = Path::new(request_string);
        let name = request_path.file_name().and_then(|n| n.to_str());
        let cleaned_name = clean_name(name.unwrap_or_default());
        String::from(cleaned_name)
    } else {
        let request_path = Path::new(request_string);
        let name = request_path.file_name().and_then(|n| n.to_str());
        let cleaned_name = clean_name(name.unwrap_or_default());
        format!("{}{}", cleaned_name, suffix)
    }
}

fn clean_name(name: &str) -> &str {
    let query_removal = Regex::new("[?].*$").unwrap();
    if query_removal.is_match(name) {
        //splitting until end of string should always return just one value: cleaned_vec[0]
        let cleaned_vec: Vec<&str> = query_removal.split(name).collect();
        cleaned_vec[0]
    } else {
        name
    }
}

fn handle_image_bytes(
    data: &mut RetrieverState,
    image_bytes: Bytes,
    file_name: &String,
) -> std::io::Result<()> {
    let file_path = Path::new(&data.export_dir).join(file_name);
    let mut buffer = File::create(file_path)?;
    //write the image bytes
    buffer.write(image_bytes.as_ref())?;
    Ok(())
}

fn log_feedback(data: &mut RetrieverState, feedback_string: String) {
    //println!("Should be adding '{}' to feedback", feedback_string);
    data.feedback.push_back(feedback_string);
}

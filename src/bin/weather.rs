use reqwest::Error;
use serde::Deserialize;

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct WeatherResponse {
    current_weather: CurrentWeather,
    current_weather_units: CurrentWeatherUnits,
    elevation: f32,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct CurrentWeather {
    interval: i32,
    is_day: i8,
    temperature: f32,
    time: String,
    winddirection: i32,
    windspeed: f32,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct CurrentWeatherUnits {
    interval: String,
    is_day: String,
    temperature: String,
    time: String,
    winddirection: String,
    windspeed: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Coordinates {
    results: Vec<Results>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Results {
    latitude: f64,
    longitude: f64,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let city = "Kathmandu";
    let geo_url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1",
        city
    );

    let coordinates: Coordinates = reqwest::get(&geo_url).await?.json().await?;
    let first_result = &coordinates.results[0];
    let lat = first_result.latitude;
    let lon = first_result.longitude;

    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current_weather=true",
        lat, lon
    );

    let data: WeatherResponse = reqwest::get(&url).await?.json().await?;

    println!("{:?}", data);
    Ok(())
}

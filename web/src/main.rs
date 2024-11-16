use core::do_things;

#[macro_use] extern crate rocket;

#[get("/")]
async fn index() -> &'static str {
    do_things().await.expect("it didn't do the thing?");

    "Hello, world!"
}

#[post("/")]
async fn go() -> &'static str {
    do_things().await.expect("it didn't do the thing?");

    "done!"
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, go])
}
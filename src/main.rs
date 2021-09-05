//like 20 pods all running at once
//in 20 different deployments

mod game;
use game::Game;
use game::MessageInterface;

use actix::{Actor, StreamHandler};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;

use actix_web::{get, Responder};

use actix_web::http::StatusCode;

use tokio::sync::Mutex;
use std::sync::Arc;


use std::time::{Duration, Instant};

use actix::prelude::*;

/*
/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(1);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);
*/


/// do websocket handshake and start `MyWebSocket` actor
#[get("/ws/")]
async fn ws_index(r: HttpRequest, stream: web::Payload, data: web::Data< Mutex<Game> > ) -> Result<HttpResponse, Error> {

    println!("getting ws request");
    
    if let Some(data) = data.lock().await.add_player(){

        //accept and upgrade to websocket and return that
        let res = ws::start( MyWs::new( data ), &r, stream );
        
        res
    }
    else{
        Ok( HttpResponse::new( StatusCode::from_u16(503).unwrap() ) )
    }

}


struct MyWs{

    //websocket interface
    data: MessageInterface,

    playerid: u8,
}

impl MyWs{

    fn new(x:  (MessageInterface, u8) ) -> MyWs{

        MyWs{
            data: x.0,
            playerid: x.1
        }
    }

}


impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;


    fn started(&mut self, ctx: &mut Self::Context) {

        ctx.text("accepted connection, send the message to set the player id");

        let mut message = Vec::new();

        if self.playerid == 1{
            message.push( 0 );
        }
        else{
            message.push( 1 );
        }

        ctx.binary( message );


        let mut data = self.data.clone();
        
        ctx.run_interval( Duration::from_millis(200) , move |act, ctx| {

            //send messages through the websocket
            if let Some(binary) = data.pop_going(){

                println!("sending binary state");

                ctx.binary( binary );
            }

            ctx.ping(b"");

        });

    }


    //if a player is removed
    //remove a player
    //and the game knows that if a player is removed, the game should restart
    fn stopped(&mut self, ctx: &mut Self::Context){

        self.data.quit();

    }

}



/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {

        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => {
                self.data.set_coming( bin.to_vec() );
            },
            _ => (),
        }


    }


}




#[get("/get_players")]
async fn get_players(  data: web::Data< Mutex<Game> > ) -> impl Responder {

    println!("requesting players");

    let data = data.lock().await;

    data.get_players_in_game().to_string()

}




#[get("/health")]
async fn health(  ) -> impl Responder {

    println!("health check");

    return "healthy".with_status(StatusCode::OK);

}





#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    println!("starting");

    
    let gamedata = web::Data::new( Mutex::new(  Game::new() ) );

    use tokio::time::{self, Duration};



    {
        let gamedata = gamedata.clone();
        let mut interval = time::interval(Duration::from_millis(33));

        //what to do if this panics?
        //i guess the most simple answer would be abort so the pod can start up again
        //but do I also panic when a player leaves? how long does it take to start up? 

        tokio::task::spawn(async move {

            loop{

                //println!("ticking");

                interval.tick().await;

                gamedata.lock().await.tick();
            }
        });
    }
    

    use actix_cors::Cors;

    

    HttpServer::new(move || {
        App::new()
            // websocket route
            .wrap( Cors::default().allow_any_origin() )
            .service(  ws_index  )
            .service(  get_players  )
            .service(  health  )
            .app_data( gamedata.clone()  )
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await

}
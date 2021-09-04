

use chessengine::ServerInterface;


//receiving and giving messages
#[derive(Clone)]
pub struct MessageInterface{

    coming: Arc<Mutex<Vec< Vec<u8> >>>,

    going: Arc<Mutex<Vec< Vec<u8> >>>,

}

impl MessageInterface{


    fn new() -> (MessageInterface, MessageInterface){


        let coming =  Arc::new(Mutex::new( Vec::new() ));

        let going = Arc::new(Mutex::new( Vec::new() ));


        let one = MessageInterface{
            coming: coming.clone(),
            going: going.clone()
        };

        let two = MessageInterface{
            coming,
            going
        };


        return (one, two);


    }

    fn set_going(&mut self, data: Vec<u8> ) {
        self.going.lock().unwrap().push( data );
    }

    fn pop_coming(&mut self) -> Option<Vec<u8>>{
        self.coming.lock().unwrap().pop()
    }

    pub fn pop_going(&mut self) -> Option< Vec<u8> >{
        self.going.lock().unwrap().pop()
    }

    pub fn set_coming(&mut self, data: Vec<u8>) {
        self.coming.lock().unwrap().push( data );
    }


}


use std::sync::Arc;
use std::sync::Mutex;



//receive message
//send message

pub struct Game{
    
    game: ServerInterface,
    
    player1websocket: Option< MessageInterface >,
    
    player2websocket: Option< MessageInterface >,
    
    ticksuntilresendstate: i32,
}


impl Game{

    pub fn is_finished(&self) -> bool{

        return false;
    }

    pub fn new()  -> Game{

        Game{
    
            game: ServerInterface::new(),
            
            player1websocket: None,
            
            player2websocket: None,
            
            ticksuntilresendstate: 100,
        }
    }


    pub fn add_player( &mut self )  -> Option<MessageInterface>{

        //if there are no players and a player is added, reset the game

        //if the players ever drop to zero

        if self.player1websocket.is_none(){

            let (toreturn, x) = MessageInterface::new();

            self.player1websocket = Some(x);

            return Some(toreturn);
        }
        else if self.player2websocket.is_none(){

            let (toreturn, x) = MessageInterface::new();

            self.player2websocket = Some(x);

            return Some(toreturn);
        }

        return None;
    }


    pub fn tick(&mut self){

        self.process_player_input();

        self.game.tick();
        
        if self.player1websocket.is_some() && self.player2websocket.is_some(){
            self.game.tick();
        }


        //self.send_state();

        
        self.ticksuntilresendstate += -1;
        if self.ticksuntilresendstate <= 0{
            self.send_state();
            self.ticksuntilresendstate = 60;
        }

        
        
    }


    fn process_player_input(&mut self){


        use std::collections::HashMap;

        let mut sockets = HashMap::new();

        if let Some(socket) = self.player1websocket.clone(){
            sockets.insert( 1, socket);
        }

        if let Some(socket) = self.player2websocket.clone(){
            sockets.insert( 2, socket);
        }



        for (player, mut socket) in sockets{

            
            let message = socket.pop_coming();

            if let Some(message) = message{

                println!("received input");

                self.game.receive_bin_input(player, message);

                self.ticksuntilresendstate = 0;
            }
        }
        
    }



    fn send_state(&mut self){

        //tick teh game forward like 5 then send?
        
        let state = self.game.get_game_string_state();

        println!("the state length {:?}", state.len() );


        if let Some(socket) = &mut self.player1websocket{
            socket.set_going( state.clone() );
        }

        if let Some(socket) = &mut self.player2websocket{
            socket.set_going( state );
        }        

    } 


    pub fn get_players_in_game(&self) -> u8{

        let mut players = 0;

        players += self.player1websocket.is_some() as u8;
        players += self.player2websocket.is_some() as u8;

        return players;
    }

}

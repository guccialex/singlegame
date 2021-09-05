

use chessengine::ServerInterface;


//receiving and giving messages
#[derive(Clone)]
pub struct MessageInterface{

    coming: Arc<Mutex<Vec< Vec<u8> >>>,

    going: Arc<Mutex<Vec< Vec<u8> >>>,

    quit: Arc<Mutex<bool>>,

}

impl MessageInterface{


    fn new() -> (MessageInterface, MessageInterface){


        let coming =  Arc::new(Mutex::new( Vec::new() ));

        let going = Arc::new(Mutex::new( Vec::new() ));


        let one = MessageInterface{
            coming: coming.clone(),
            going: going.clone(),
            quit: Arc::new( Mutex::new( false )  ),
            
        };



        return (one.clone(), one.clone());


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


    pub fn quit(&mut self){
        *self.quit.lock().unwrap() = true;
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


    pub fn add_player( &mut self )  -> Option<(MessageInterface, u8)>{


        if self.player1websocket.is_none(){

            let (toreturn, x) = MessageInterface::new();

            self.player1websocket = Some(x);

            return Some( (toreturn, 1) );
        }
        else if self.player2websocket.is_none(){

            let (toreturn, x) = MessageInterface::new();

            self.player2websocket = Some(x);

            return Some( (toreturn,2) );
        }

        return None;
    }
    

    fn reset(&mut self){
        *self = Game::new();
    }


    pub fn tick(&mut self){

        self.process_player_input();

        self.game.tick();
        
        if self.are_both_players_connected(){
            self.game.tick();
        }

        
        self.ticksuntilresendstate += -1;
        if self.ticksuntilresendstate <= 0{
            self.send_state();
            self.ticksuntilresendstate = 40;
        }


        if self.is_finished(){
            self.reset();
        }


        //if either player sent a "quit" message, remove them from the game
        //and end the game if its started
        {

            if let Some(message) = &self.player1websocket{
                if *message.quit.lock().unwrap() == true{

                    self.reset();
                    //self.player1websocket = None;
                }
            }
    
            if let Some(message) = &self.player2websocket{
                if *message.quit.lock().unwrap() == true{
    
                    self.reset();
                    //self.player2websocket = None;
                }
            }

        }

    }


    fn are_both_players_connected(&self) -> bool{

        if self.player1websocket.is_some() && self.player2websocket.is_some(){
    
            return true;
        }

        return false;
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

                self.game.receive_bin_input(player, message);

                self.ticksuntilresendstate = 0;
            }
        }
        
    }



    fn send_state(&mut self){

        let state = self.game.get_game_string_state();

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

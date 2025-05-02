#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, Address, symbol_short};

// Lottery ticket data structure
#[contracttype]
#[derive(Clone)]
pub struct LotteryTicket {
    pub ticket_id: u64,
    pub owner: Address,
    pub timestamp: u64,
}

// Lottery state data structure
#[contracttype]
#[derive(Clone)]
pub struct LotteryState {
    pub active: bool,
    pub ticket_count: u64,
    pub ticket_price: u64,
    pub pot_amount: u64,
    pub last_winner: Address,
    pub last_win_amount: u64,
}

const LOTTERY_STATE: Symbol = symbol_short!("LOT_STATE");
const TICKET_COUNT: Symbol = symbol_short!("TKT_COUNT");

// Mapping ticket ID to ticket data
#[contracttype]
pub enum TicketMap {
    Ticket(u64)
}

#[contract]
pub struct CryptoLotteryContract;

#[contractimpl]
impl CryptoLotteryContract {
    // Initialize a new lottery round
    pub fn initialize(env: Env, ticket_price: u64) -> LotteryState {
        let state = LotteryState {
            active: true,
            ticket_count: 0,
            ticket_price: ticket_price,
            pot_amount: 0,
            last_winner: env.current_contract_address(),
            last_win_amount: 0,
        };
        
        env.storage().instance().set(&LOTTERY_STATE, &state);
        env.storage().instance().set(&TICKET_COUNT, &0);
        
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Lottery initialized with ticket price: {}", ticket_price);
        
        state
    }
    
    // Buy a lottery ticket
    pub fn buy_ticket(env: Env, buyer: Address) -> u64 {
        let mut state = Self::get_lottery_state(env.clone());
        
        if !state.active {
            log!(&env, "Lottery is not active");
            panic!("Lottery is not active");
        }
        
        // In a real implementation, payment processing would go here
        // For simplicity, we're just recording the ticket purchase
        
        let mut ticket_count = env.storage().instance().get(&TICKET_COUNT).unwrap_or(0);
        ticket_count += 1;
        
        let ticket = LotteryTicket {
            ticket_id: ticket_count,
            owner: buyer.clone(),
            timestamp: env.ledger().timestamp(),
        };
        
        env.storage().instance().set(&TicketMap::Ticket(ticket_count), &ticket);
        env.storage().instance().set(&TICKET_COUNT, &ticket_count);
        
        // Update lottery state
        state.ticket_count += 1;
        state.pot_amount += state.ticket_price;
        env.storage().instance().set(&LOTTERY_STATE, &state);
        
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Ticket {} purchased by {}", ticket_count, buyer);
        
        ticket_count
    }
    
    // Draw a winner for the lottery
    pub fn draw_winner(env: Env) -> Address {
        let mut state = Self::get_lottery_state(env.clone());
        
        if !state.active || state.ticket_count == 0 {
            log!(&env, "Lottery is not active or no tickets purchased");
            panic!("Lottery is not active or no tickets purchased");
        }
        
        // Generate a pseudo-random number based on timestamp
        // Note: This is not secure for a real lottery, just for demo purposes
        let timestamp = env.ledger().timestamp();
        let ticket_count = state.ticket_count;
        let winning_ticket_id = (timestamp % ticket_count) + 1;
        
        // Get the winning ticket
        let ticket: LotteryTicket = env.storage().instance().get(&TicketMap::Ticket(winning_ticket_id))
            .unwrap_or_else(|| panic!("Ticket not found"));
        
        let winner = ticket.owner;
        
        // Update lottery state
        state.active = false;
        state.last_winner = winner.clone();
        state.last_win_amount = state.pot_amount;
        state.pot_amount = 0;
        
        env.storage().instance().set(&LOTTERY_STATE, &state);
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Winner drawn: {} won {} tokens", winner, state.last_win_amount);
        
        winner
    }
    
    // Get current lottery state
    pub fn get_lottery_state(env: Env) -> LotteryState {
        env.storage().instance().get(&LOTTERY_STATE).unwrap_or(LotteryState {
            active: false,
            ticket_count: 0,
            ticket_price: 0,
            pot_amount: 0,
            last_winner: env.current_contract_address(),
            last_win_amount: 0,
        })
    }
}
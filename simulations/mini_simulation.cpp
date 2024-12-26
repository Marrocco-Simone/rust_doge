/*starting
  pub const STARTING_QUANTITY: f32 = 0.0;
  pub const STARTING_CAPITAL: f32 = 1_000_000.0;

  pub const DEFAULT_EUR_YEN_EXCHANGE_RATE: f32 = 143.615;
  pub const DEFAULT_EUR_USD_EXCHANGE_RATE: f32 = 1.03576;
  pub const DEFAULT_EUR_YUAN_EXCHANGE_RATE: f32 = 7.3599;
*/

#include <stdio.h>      /* printf, scanf */
#include <stdlib.h>     /* srand, rand */
#include <time.h>       /* time */

float const starting_capital = 1e+6;
int const N_CURRENCIES = 4;
int const EUR = 0;
int const USD = 1;
int const YEN = 2;
int const YUAN = 3;

// float const EUR_USD_EXCHANGE_RATE = 1.03576;
// float const EUR_YEN_EXCHANGE_RATE = 143.615;
// float const EUR_YUAN_EXCHANGE_RATE = 7.3599;

// measure unit GOOD/EUR
float const EXCHANGE_RATE[] = {
  1, 1.03576, 143.615, 7.3599
};

char* const GOOD_NAME[] = {
  (char*)"eur", (char*)"usd", (char*)"yen", (char*)"yuan"
};

//NOT IMPORTANT, JUST LOGGING
void logMarketAndTrader(float* market, float* trader, float interest) {  
  printf("interest: \033[0;32m%.4f\033[0m\n", interest);

  printf("market: ");
  for (int GOOD=0; GOOD < N_CURRENCIES; GOOD++) 
    printf("\033[0;32m%.2f\033[0m %s, ", market[GOOD], GOOD_NAME[GOOD]);
  printf("\n");
  
  printf("trader: ");
  for (int GOOD=0; GOOD < N_CURRENCIES; GOOD++) 
    printf("\033[0;32m%.2f\033[0m %s, ", trader[GOOD], GOOD_NAME[GOOD]);
  printf("\n");
}

//NOT IMPORTANT, JUST LOGGING
void logTransaction(float q_good, int GOOD, bool is_buy, float euros_to_pay) {
  if (!euros_to_pay) return;

  int is_buy_factor = 0;
  if (is_buy) is_buy_factor = 1;
  else is_buy_factor = -1;

  if (is_buy) printf("trader buyed ");
  else printf("trader sold ");
  printf("\033[0;32m%.2f\033[0m %s with \033[0;32m%.2f\033[0m eur\n", q_good, GOOD_NAME[GOOD], euros_to_pay);

  float standard_change = q_good / EXCHANGE_RATE[GOOD];
  float difference_gain = (euros_to_pay - standard_change) * is_buy_factor;
  if (difference_gain > 0)
    printf("standard change would be \033[0;32m%.2f\033[0m, for a difference gain of \033[0;32m%.2f\033[0m\n", standard_change, difference_gain);
  else printf("standard change would be \033[0;31m%.2f\033[0m, for a difference gain of \033[0;31m%.2f\033[0m\n", standard_change, difference_gain);
}

/*
 !IMPLEMENT: in rust but not like this
 what we need is a function that tells us if the market has the quantity that has to be sold
 or enough euro to buy the good from the trader.

 here the checks are different, since I needed to check for both the market and the trader
*/
bool is_transaction_possible(float* market, float* trader, float q_good, int GOOD, float euros_to_pay, bool is_buy) {
  if (is_buy) {
    if (market[GOOD] < q_good) return false;
    if (trader[EUR] < euros_to_pay) return false;
  } else {
    if (trader[GOOD] < q_good) return false;
    if (market[EUR] < euros_to_pay) return false;
  }
  return true;
}

/*
  !IMPLEMENT: in rust
  q_good is the quantity of the good that we want to move, GOOD is the type of the good.
  is_buy tells me if the transaction is "the trader wants to buy this good" or
  "the trader wants to sell to the market this good"

  this function also makes the transaction, but in rust we just need a function that gives us the price.
*/
float transaction(float* market, float* trader, float interest, float q_good, int GOOD, bool is_buy) {
  /*
    if the trader wants to trade eur, the exchange should be 1:1 and basically give back the same amount
    !IMPORTANT: people may lock eur with eur to exploit some price changes. We should refuse transactions EUR to EUR
    here we return 0 since this function returns the euro cost of the transaction, but in rust won't need that
  */
  if (GOOD == EUR) return 0;

  // factor to make the function work for both buying and selling, since this two operations differ from some +- signs
  int is_buy_factor = 0;
  if (is_buy) is_buy_factor = 1;
  else is_buy_factor = -1;
  
  //!calculate the euros to pay for the transaction (the trader must pay us this to buy the goods or we buy the goods from the trader for this price). THIS SHOULD BE IMPLEMENTED IN RUST
  float change_rate = market[EUR] / (market[GOOD] - (q_good * is_buy_factor));
  float euros_to_pay = q_good * change_rate;
  euros_to_pay *= 1 + (interest * is_buy_factor);

  // if the transaction is not possible to execute, abort
  if (!is_transaction_possible(market, trader, q_good, GOOD, euros_to_pay, is_buy)) return 0;

  // execute the transaction by moving the moneys
  market[EUR] += euros_to_pay * is_buy_factor;
  trader[EUR] -= euros_to_pay * is_buy_factor;
  market[GOOD] -= q_good * is_buy_factor;
  trader[GOOD] += q_good * is_buy_factor;

  // for clarity of what we're passing
  float euros_payed = euros_to_pay;
  return euros_payed;
}

// made specific to be more clear, but we should implement the generic version made in transaction(). for full explanation see transaction()
float traderBuys(float* market, float* trader, float interest, float q_good, int GOOD) {
  if (GOOD == EUR) return 0;
  
  float change_rate = market[EUR] / (market[GOOD] - q_good);
  float euros_to_pay = q_good * change_rate;
  euros_to_pay *= 1 + interest;

  if (!is_transaction_possible(market, trader, q_good, GOOD, euros_to_pay, true)) return 0;

  market[EUR] += euros_to_pay;
  trader[EUR] -= euros_to_pay;
  market[GOOD] -= q_good;
  trader[GOOD] += q_good;

  float euros_payed = euros_to_pay;
  return euros_payed;

  // using the generic function transaction
  // return transaction(market, trader, interest, q_good, GOOD, true);
}

// made specific to be more clear, but we should implement the generic version made in transaction(). for full explanation see transaction()
float traderSells(float* market, float* trader, float interest, float q_good, int GOOD) {
  if (GOOD == EUR) return 0;
  
  float change_rate = market[EUR] / (market[GOOD] + q_good);
  float euros_to_pay = q_good * change_rate;
  euros_to_pay *= 1 - interest;

  if (!is_transaction_possible(market, trader, q_good, GOOD, euros_to_pay, false)) return 0;

  market[EUR] -= euros_to_pay;
  trader[EUR] += euros_to_pay;
  market[GOOD] += q_good;
  trader[GOOD] -= q_good;

  float euros_payed = euros_to_pay;
  return euros_payed;

  // using the generic function transaction
  // return transaction(market, trader, interest, q_good, GOOD, false);
}

float min(float a, float b) {
  if (a < b) return a;
  return b;
}

int const CHANGE_DAYS_LIMIT = 100;
float const IMPORT_TAX = 0.25;
int SHORTAGE_PROBABILITY_PERCENT = 5;
// todo: should test this value, maybe we can lower it
int const CAREFUL_FRACTION = 8;
// market becomes an importer of the good after this limit. We use an eight of the starting capital for each (so it should be half of the starting value)
float const GOODS_CAREFUL_VALUE[] = {
  starting_capital / CAREFUL_FRACTION,
  (starting_capital / CAREFUL_FRACTION) * EXCHANGE_RATE[USD],
  (starting_capital / CAREFUL_FRACTION) * EXCHANGE_RATE[YEN],
  (starting_capital / CAREFUL_FRACTION) * EXCHANGE_RATE[YUAN],
};

// !IMPLEMENT: we have too little of this good, so we need to import it. It is a GOOD KIND
int searchForGoodToRefill(
  float* market, bool* is_good_importer, int* days_since_last_change, int* days_after_shortage
) {
  int good_to_refill = -1;

  for (int GOOD=0; GOOD < N_CURRENCIES; GOOD++) {
    if (market[GOOD] > GOODS_CAREFUL_VALUE[GOOD]) {
      // the good does not need to be refilled

      if (
        market[GOOD] > GOODS_CAREFUL_VALUE[GOOD] * 2 && // here we use a bigger GOODS_CAREFUL_VALUE to avoid constantly switching back and forth
        is_good_importer[GOOD] && 
        days_since_last_change[GOOD] >= CHANGE_DAYS_LIMIT
      ) {
        // reset the good as exported if it is currently of type imported
        is_good_importer[GOOD] = false;
      }

      continue;
    }

    // we need more of the good since is below the careful_value

    if (!is_good_importer[GOOD]) {
      // if the good is not imported, we need to change it first

      if (days_since_last_change[GOOD] < CHANGE_DAYS_LIMIT) continue; // we can't do anything yet, 100 days haven't been passed

      is_good_importer[GOOD] = true;
    }

    if (days_after_shortage[GOOD] < CHANGE_DAYS_LIMIT) continue; // market is in shortage

    // we can import the good, but we only import the one we need the most
    if (good_to_refill < 0) {
      // this is the first good we found out we need to import
      good_to_refill = GOOD;
      continue;
    }

    // save only the good we have the least
    float this_good_quantity = market[GOOD] / EXCHANGE_RATE[GOOD];
    float good_to_refill_quantity = market[good_to_refill] / EXCHANGE_RATE[good_to_refill];
    if (this_good_quantity < good_to_refill_quantity) good_to_refill = GOOD;
  }

  return good_to_refill;
}

// !IMPLEMENT: find the good we have the most that is not an importer and that is over the careful_limit * 2
int searchMostAbundantGood(
  float* market, bool* is_good_importer, int* days_since_last_change, int* days_after_shortage
) {
  int most_abundant_good = -1;

  for (int GOOD=0; GOOD < N_CURRENCIES; GOOD++) {
    if (is_good_importer[GOOD]) continue; // remember that all possible goods that could have been exporters have already been transformed into them before at the start of the previous cycle, so here we don't have any good that is importer but could be converted in an exporter
    if (market[GOOD] <= GOODS_CAREFUL_VALUE[GOOD] * 2) continue; // not safe enough

    // this is the first good we found out we need to import
    if (most_abundant_good < 0) {
      most_abundant_good = GOOD;
      continue;
    }

    // save only the good we have the most
    float this_good_quantity = market[GOOD] / EXCHANGE_RATE[GOOD];
    float most_abundant_good_quantity = market[most_abundant_good] / EXCHANGE_RATE[most_abundant_good];
    if (this_good_quantity > most_abundant_good_quantity) most_abundant_good = GOOD;
  }

  return most_abundant_good;
}

// !IMPLEMENT:, arguments are explained in simulation()
void refillGoods(
  float* market, bool* is_good_importer, int* days_since_last_change, int* days_after_shortage
) {
  // for (int GOOD=0; GOOD < N_CURRENCIES; GOOD++): means we cycle every GOOD KIND
  // here, something[GOOD] means that we have a struct something with 4 fields, one for each GOOD KIND

  // next day
  for (int GOOD=0; GOOD < N_CURRENCIES; GOOD++) {
    days_since_last_change[GOOD]++;
    days_after_shortage[GOOD]++;
  }

  // we have too little of this good, so we need to import it. It is a GOOD KIND
  int good_to_refill = searchForGoodToRefill(market, is_good_importer, days_since_last_change, days_after_shortage);
  if (good_to_refill < 0) return; // all fine, there is no good to refill
  
  // simulate a probability of shortage within a SHORTAGE_PROBABILITY_PERCENT
  int shortage_probability = rand() % 100;
  if (shortage_probability < SHORTAGE_PROBABILITY_PERCENT) {
    // reset days_after_shortage for this good
    days_after_shortage[good_to_refill] = 0;
    
    // logging, don't care
    printf("\033[0;31m!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!\n");
    printf("MARKET SHORTAGE OF %s\033[0m\n", GOOD_NAME[good_to_refill]);

    return;
  }

  int most_abundant_good = searchMostAbundantGood(market, is_good_importer, days_since_last_change, days_after_shortage);
  if (most_abundant_good < 0) return; // there is no good we can use to exchange

  /*
    we want to trade most_abundant_good for good_to_refill so that 
    market[good_to_refill] becomes greater than GOODS_CAREFUL_VALUE[good_to_refill], 
    while market[most_abundant_good] remains greater than GOODS_CAREFUL_VALUE[most_abundant_good]
  */
  float good_to_refill_q_needed =
    GOODS_CAREFUL_VALUE[good_to_refill] - market[good_to_refill];
  if (good_to_refill_q_needed < 0) return; // should not happen

  float most_abundant_good_q_available =
    market[most_abundant_good] - GOODS_CAREFUL_VALUE[most_abundant_good];
  if (most_abundant_good_q_available < 0) return; // can satisfy request

  // we can only exchange the min between the two values
  float most_abundant_good_q_to_exchange =
    min(good_to_refill_q_needed, most_abundant_good_q_available);

  // change "most_abundant_good_q_to_exchange" in EUR
  float most_abundant_good_q_to_exchange_in_eur = 
    most_abundant_good_q_to_exchange / EXCHANGE_RATE[most_abundant_good];

  // convert "most_abundant_good_q_to_exchange_in_eur" in the "good_to_refill" kind and then apply the tax (so multiply it by 75%)
  float good_to_refill_q_received = 
    most_abundant_good_q_to_exchange_in_eur * EXCHANGE_RATE[good_to_refill] * (1 - IMPORT_TAX);

  // transaction
  market[most_abundant_good] -= most_abundant_good_q_to_exchange;
  market[good_to_refill] += good_to_refill_q_received;
  
  // logging, don't care
  float good_to_refill_q_received_in_eur = 
    good_to_refill_q_received / EXCHANGE_RATE[good_to_refill];
  float loss = 
    most_abundant_good_q_to_exchange_in_eur - good_to_refill_q_received_in_eur;
  printf("\033[0;31m!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!\n");
  printf("changed %.2f %s with %.2f %s, with a loss of %.2f eur\033[0m\n",
    most_abundant_good_q_to_exchange, GOOD_NAME[most_abundant_good],
    good_to_refill_q_received, GOOD_NAME[good_to_refill],
    loss
  );
}

// simulate a market and a trader that make transactions
void simulation(float interest, int n_transactions) {
  // our market with the starting values
  // the idea is dividing the starting capital in four and then converting each part in its currency
  float market[] = {
    starting_capital / N_CURRENCIES,
    (starting_capital / N_CURRENCIES) * EXCHANGE_RATE[USD],
    (starting_capital / N_CURRENCIES) * EXCHANGE_RATE[YEN],
    (starting_capital / N_CURRENCIES) * EXCHANGE_RATE[YUAN],
  };

  // the market can create currency by being an importer or an exporter of a good (for each good). We save here our situation for each good
  bool is_good_importer[] = { false, false, false, false };
  // the market can change from importer to exporter and viceversa for a good every 100 days or more. We save here the days passed since our last change (if >100, we can change)
  int days_since_last_change[] = { 
    CHANGE_DAYS_LIMIT, CHANGE_DAYS_LIMIT, CHANGE_DAYS_LIMIT, CHANGE_DAYS_LIMIT 
  };
  // there is a possibility of 5% that the good import/export breaks and that we can't use it for 100 days. We save here the days passed by the last shortage, the idea is that we can't import/export a good if its value here is below 100
  int days_after_shortage[] = { 
    CHANGE_DAYS_LIMIT, CHANGE_DAYS_LIMIT, CHANGE_DAYS_LIMIT, CHANGE_DAYS_LIMIT 
  };

  // random starting values of the trader
  float trader[] = {
    starting_capital,
    starting_capital * EXCHANGE_RATE[USD],
    starting_capital * EXCHANGE_RATE[YEN],
    starting_capital * EXCHANGE_RATE[YUAN],
  };

  logMarketAndTrader(market, trader, interest);
  printf("\n");

  // simulate n transactions
  for (int remaining = n_transactions; remaining > 0; remaining--) {
    // random value of the good
    int rand_value = rand() % int(starting_capital/N_CURRENCIES);
    // random good kind (here a good is determined by a integer, see at the top of the file)
    int rand_good = rand() % N_CURRENCIES;
    int is_buy = rand() % 2;
    float euros_payed = 0;

    //!IMPORTANT: TRANSACTION
    if (is_buy)
      euros_payed = traderBuys(market, trader, interest, rand_value, rand_good);
    else euros_payed = traderSells(market, trader, interest, rand_value, rand_good);
    
    //!IMPORTANT: CHECK FOR REFILL
    // must be done every day/tick
    refillGoods(market, is_good_importer, days_since_last_change, days_after_shortage);

    // log only if there was a transaction
    if (euros_payed) {
      logTransaction( rand_value, rand_good, is_buy, euros_payed);
      logMarketAndTrader(market, trader, interest);
      printf("\n");
    }
  }
}

int main(int argc, char** argv) {
  srand(time(0));
  // total number of transactions to simulate
  int n_transactions = 1000;
  // extra interest at each transaction, so that we may gain from it
  float interest = 0.01;

  simulation(interest, n_transactions);
    
  printf("simulation finished\n\n");
}

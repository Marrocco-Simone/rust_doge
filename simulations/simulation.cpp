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

#define FOR_EACH_GOOD for (int GOOD=0; GOOD < N_CURRENCIES; GOOD++)

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

void logMarketAndTrader(bool log_debug, float* market, float* trader, float interest) {
  if(!log_debug) return;
  
  printf("interest: \033[0;32m%.4f\033[0m\n", interest);

  printf("market: ");
  FOR_EACH_GOOD 
    printf("\033[0;32m%.2f\033[0m %s, ", market[GOOD], GOOD_NAME[GOOD]);
  printf("\n");
  
  printf("trader: ");
  FOR_EACH_GOOD 
    printf("\033[0;32m%.2f\033[0m %s, ", trader[GOOD], GOOD_NAME[GOOD]);
  printf("\n");
}

void logTransaction(bool log_debug, float q_good, int GOOD, bool is_buy, float euros_to_pay) {
  if(!log_debug) return;
  
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

void logGain(bool log_debug, float start, float end, float gain_percenteage) {
  if(!log_debug) return;
  
  float difference = end - start;
  if (difference == 0) return;
  
  if(difference > 0) {
    printf("market started with \033[0;32m%.2f\033[0m and ended with \033[0;32m%.2f\033[0m \n", start, end);
    printf("market gain: \033[0;32m%.5f\033[0m%% (total: \033[0;32m%.2f\033[0m eur) \n", gain_percenteage, difference);
  } else {
    printf("market started with \033[0;31m%.2f\033[0m and ended with \033[0;31m%.2f\033[0m \n", start, end);
    printf("market gain: \033[0;31m%.5f\033[0m%% (total: \033[0;31m%.2f\033[0m eur) \n", gain_percenteage, difference);
  }
  printf("\n");
}

FILE *transactions_json_file;
FILE *simulations_json_file;
FILE *market_json_file;
FILE *trader_json_file;

bool first_json_transaction = true;
void JsonTransaction (bool save_json_transactions, bool is_buy, float q_good, int GOOD, float euros_payed, float start, float end, float gain_percenteage) {
  if (!save_json_transactions) return;

  if (first_json_transaction) first_json_transaction = false;
  else fprintf(transactions_json_file, ",\n");

  float standard_change = q_good / EXCHANGE_RATE[GOOD];
  fprintf(transactions_json_file, 
    "{\"is_buy\": %d, \"quantity\": %.2f, \"good_type\": \"%s\", \"euros_payed\": %.2f, \"standard_change\": %.2f, \"market_start\": %.2f, \"market_end\": %.2f, \"gain_percenteage\": %.5f}", 
    is_buy, q_good, GOOD_NAME[GOOD], euros_payed, standard_change, start, end, gain_percenteage
  );
}

bool first_json_simulation = true;
void JsonSimulation (bool save_json_simulations, float interest, int n_transactions, float gain_percentages_mean) {
  if (!save_json_simulations) return;

  if (first_json_simulation) first_json_simulation = false;
  else fprintf(simulations_json_file, ",\n");
  
  fprintf(simulations_json_file, 
    "{\"interest\": %.3f, \"n_transactions\": %d, \"gain_percentages_mean\": %.2f}", 
    interest, n_transactions, gain_percentages_mean
  );
}

bool first_json_market = true;
void JsonMarket (bool save_json_market_trader, float* market, int index) {
  if (!save_json_market_trader) return;

  if (first_json_market) first_json_market = false;
  else fprintf(market_json_file, ",\n");
  
  fprintf(market_json_file, "{\"index\": %d, ", index);
  FOR_EACH_GOOD {
    fprintf(market_json_file, "\"%s\": %.2f", GOOD_NAME[GOOD], market[GOOD]);
    if (GOOD != N_CURRENCIES -1) fprintf(market_json_file, ", ");
  }
  fprintf(market_json_file, "}");
}

bool first_json_trader = true;
void JsonTrader (bool save_json_market_trader, float* trader, int index) {
  if (!save_json_market_trader) return;

  if (first_json_trader) first_json_trader = false;
  else fprintf(trader_json_file, ",\n");
  
  fprintf(trader_json_file, "{\"index\": %d, ", index);
  FOR_EACH_GOOD {
    fprintf(trader_json_file, "\"%s\": %.2f", GOOD_NAME[GOOD], trader[GOOD]);
    if (GOOD != N_CURRENCIES -1) fprintf(trader_json_file, ", ");
  }
  fprintf(trader_json_file, "}");
}

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

float transaction(float* market, float* trader, float interest, float q_good, int GOOD, bool is_buy) {
  if (GOOD == EUR) return 0;

  int is_buy_factor = 0;
  if (is_buy) is_buy_factor = 1;
  else is_buy_factor = -1;
  
  float change_rate = market[EUR] / (market[GOOD] - (q_good * is_buy_factor));
  float euros_to_pay = q_good * change_rate;
  euros_to_pay *= 1 + (interest * is_buy_factor);

  if (!is_transaction_possible(market, trader, q_good, GOOD, euros_to_pay, is_buy)) return 0;

  market[EUR] += euros_to_pay * is_buy_factor;
  trader[EUR] -= euros_to_pay * is_buy_factor;
  market[GOOD] -= q_good * is_buy_factor;
  trader[GOOD] += q_good * is_buy_factor;

  float euros_payed = euros_to_pay;
  return euros_payed;
}

float buy(float* market, float* trader, float interest, float q_good, int GOOD) {
  float euros_payed = transaction(market, trader, interest, q_good, GOOD, true);
  return euros_payed;
}

float sell(float* market, float* trader, float interest, float q_good, int GOOD) {
  float euros_payed = transaction(market, trader, interest, q_good, GOOD, false);
  return euros_payed;
}

float convertMarketToEUR(float* market) {
  float eur = 0;
  FOR_EACH_GOOD eur += market[GOOD] / EXCHANGE_RATE[GOOD];
  return eur;
}

float calculateGain(float start, float end) {
  float difference = end - start;
  float gain_percenteage = difference / start*100;
  return gain_percenteage;
}

float mean(float *arr, int dim) {
  float sum = 0;
  for (int i = 0; i < dim; i++) 
    sum += arr[i]/dim;
  return sum;
}

float min(float a, float b) {
  if (a < b) return a;
  return b;
}

/*int const MIN_FRACTION = 16;
float const GOODS_MIN_VALUE[] = {
  starting_capital / MIN_FRACTION,
  (starting_capital / MIN_FRACTION) * EXCHANGE_RATE[USD],
  (starting_capital / MIN_FRACTION) * EXCHANGE_RATE[YEN],
  (starting_capital / MIN_FRACTION) * EXCHANGE_RATE[YUAN],
}; */

//market becomes an importer of the good after this limit
int const CAREFUL_FRACTION = 8;
float const GOODS_CAREFUL_VALUE[] = {
  starting_capital / CAREFUL_FRACTION,
  (starting_capital / CAREFUL_FRACTION) * EXCHANGE_RATE[USD],
  (starting_capital / CAREFUL_FRACTION) * EXCHANGE_RATE[YEN],
  (starting_capital / CAREFUL_FRACTION) * EXCHANGE_RATE[YUAN],
};
int const CHANGE_DAYS_LIMIT = 100;
float const IMPORT_TAX = 0.25;
int SHORTAGE_PROBABILITY_PERCENT = 5;

// find the good we have the least that is not an exporter and that is below the careful_limit
int searchGoodToRefill(float* market, bool* is_good_importer, int* days_since_last_change, int* days_after_shortage, bool is_refill_failure) {
  int good_to_refill = -1;

  FOR_EACH_GOOD {
    // the good does not need to be refilled
    if (market[GOOD] > GOODS_CAREFUL_VALUE[GOOD]) {
      // reset the good as exported if it is imported
      if (
        market[GOOD] > GOODS_CAREFUL_VALUE[GOOD] * 2 && 
        is_good_importer[GOOD] && 
        days_since_last_change[GOOD] >= CHANGE_DAYS_LIMIT
      ) {
        is_good_importer[GOOD] = false;
      }
      continue;
    }
    // we need more of the good since is below the careful_value

    // if the good is not imported, we need to change it first
    if (!is_good_importer[GOOD]) {
      // we can't do anything yet
      if (days_since_last_change[GOOD] < CHANGE_DAYS_LIMIT) continue; 
      is_good_importer[GOOD] = true;
    }
    // we can import the good, but we only import the one we need the most

    // market is in shortage
    if (days_after_shortage[GOOD] < CHANGE_DAYS_LIMIT) continue; 

    // this is the first good we found out we need to import
    if (good_to_refill < 0) {
      good_to_refill = GOOD;
      continue;
    }

    float this_good_quantity = market[GOOD] / EXCHANGE_RATE[GOOD];
    float good_to_refill_quantity = market[good_to_refill] / EXCHANGE_RATE[good_to_refill];
    if (this_good_quantity < good_to_refill_quantity) good_to_refill = GOOD;
  }

  return good_to_refill;
}

// find the good we have the most that is not an importer and that is over the careful_limit * 2
int searchMostAbundantGood(float* market, bool* is_good_importer, int* days_since_last_change, int* days_after_shortage, bool is_refill_failure) {
  int most_abundant_good = -1;

  FOR_EACH_GOOD {
    if (is_good_importer[GOOD]) continue; // remember that all possible goods that could have been exporters have already been transformed into them before at the start of the previous cycle, so here we don't have any good that is importer but could be converted in an exporter
    if (market[GOOD] <= GOODS_CAREFUL_VALUE[GOOD] * 2) continue;

    // this is the first good we found out we need to import
    if (most_abundant_good < 0) {
      most_abundant_good = GOOD;
      continue;
    }

    float this_good_quantity = market[GOOD] / EXCHANGE_RATE[GOOD];
    float most_abundant_good_quantity = market[most_abundant_good] / EXCHANGE_RATE[most_abundant_good];
    if (this_good_quantity > most_abundant_good_quantity) most_abundant_good = GOOD;
  }

  return most_abundant_good;
}

//check if good is under a limit
void refillGoods(float* market, bool* is_good_importer, int* days_since_last_change, int* days_after_shortage, bool log_debug, bool is_refill_failure) {
  FOR_EACH_GOOD {
    days_since_last_change[GOOD]++;
    days_after_shortage[GOOD]++;
  }

  int good_to_refill = searchGoodToRefill(
    market, is_good_importer, days_since_last_change, days_after_shortage, is_refill_failure
  );
  if (good_to_refill < 0) return; // all fine, there is no good to refill
  
  // simulate a probability of shortage within a SHORTAGE_PROBABILITY_PERCENT
  if (is_refill_failure) {
    int shortage_probability = rand() % 100;
    if (shortage_probability < SHORTAGE_PROBABILITY_PERCENT) {
      days_after_shortage[good_to_refill] = 0;
      if(log_debug) {
        printf("\033[0;31m!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!\n");
        printf("MARKET SHORTAGE OF %s\033[0m\n", GOOD_NAME[good_to_refill]);
      }
      return;
    }
  }

  int most_abundant_good = searchMostAbundantGood(
    market, is_good_importer, days_since_last_change, days_after_shortage, is_refill_failure
  );
  if (most_abundant_good < 0) return; // there is no good we can use to exchange

  // we want to trade most_abundant_good for good_to_refill so that 
  // market[good_to_refill] > GOODS_CAREFUL_VALUE[good_to_refill], 
  // while market[most_abundant_good] remains > GOODS_CAREFUL_VALUE[most_abundant_good]
  float good_to_refill_q_needed = 
    GOODS_CAREFUL_VALUE[good_to_refill] - market[good_to_refill];
  if (good_to_refill_q_needed < 0) return; // should not happen
  float most_abundant_good_q_available = 
    market[most_abundant_good] - GOODS_CAREFUL_VALUE[most_abundant_good];
  if (most_abundant_good_q_available < 0) return; // cannot satisfy request

  //! ANDREBBERO CONVERTITI IN EURO
  float most_abundant_good_q_to_exchange = 
    min(good_to_refill_q_needed, most_abundant_good_q_available);

  float most_abundant_good_q_to_exchange_in_eur = 
    most_abundant_good_q_to_exchange / EXCHANGE_RATE[most_abundant_good];
  float good_to_refill_q_received = 
    most_abundant_good_q_to_exchange_in_eur * EXCHANGE_RATE[good_to_refill] * (1 - IMPORT_TAX);

  market[most_abundant_good] -= most_abundant_good_q_to_exchange;
  market[good_to_refill] += good_to_refill_q_received;
  
  if(log_debug) {
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
}

float simulation(float interest, int n_transactions, bool log_debug, bool save_json_simulations, bool save_json_market_trader, bool save_json_transactions, bool is_refill, bool is_refill_failure) {
  float market[] = {
    starting_capital / N_CURRENCIES,
    (starting_capital / N_CURRENCIES) * EXCHANGE_RATE[USD],
    (starting_capital / N_CURRENCIES) * EXCHANGE_RATE[YEN],
    (starting_capital / N_CURRENCIES) * EXCHANGE_RATE[YUAN],
  };

  bool is_good_importer[] = { 
    false, false, false, false 
  };
  //! QUESTI ERANO ZERO. PERCHE'
  int days_since_last_change[] = { 
    CHANGE_DAYS_LIMIT, CHANGE_DAYS_LIMIT, CHANGE_DAYS_LIMIT, CHANGE_DAYS_LIMIT 
  };
  int days_after_shortage[] = { 
    CHANGE_DAYS_LIMIT, CHANGE_DAYS_LIMIT, CHANGE_DAYS_LIMIT, CHANGE_DAYS_LIMIT 
  };

  float trader[] = {
    starting_capital,
    starting_capital * EXCHANGE_RATE[USD],
    starting_capital * EXCHANGE_RATE[YEN],
    starting_capital * EXCHANGE_RATE[YUAN],
  };

  logMarketAndTrader(log_debug, market, trader, interest);
  if (log_debug) printf("\n");
  JsonMarket(save_json_market_trader, market, 0);
  JsonTrader(save_json_market_trader, trader, 0);
  float total_start = convertMarketToEUR(market);

  for (int remaining = n_transactions; remaining > 0; remaining--) {
    float start = convertMarketToEUR(market);

    int rand_good = rand() % N_CURRENCIES;
    int rand_value = rand() % int(starting_capital/N_CURRENCIES);
    int is_buy = rand() % 2;
    float euros_payed = transaction(market, trader, interest, rand_value, rand_good, is_buy);
    
    if (is_refill) 
      refillGoods(market, is_good_importer, days_since_last_change, days_after_shortage, log_debug, is_refill_failure);

    float end = convertMarketToEUR(market);
    float gain_percenteage = calculateGain(start, end);

    JsonMarket(save_json_market_trader, market, n_transactions - remaining + 1);
    JsonTrader(save_json_market_trader, trader, n_transactions - remaining + 1);
    if (euros_payed) {
      logTransaction(log_debug, rand_value, rand_good, is_buy, euros_payed);
      logMarketAndTrader(log_debug, market, trader, interest);
      logGain(log_debug, start, end, gain_percenteage);
      JsonTransaction(save_json_transactions, is_buy, rand_value, rand_good, euros_payed, start, end, gain_percenteage);
    }
  }

  float total_end = convertMarketToEUR(market);
  float gain_percenteage = calculateGain(total_start, total_end);
  if (log_debug) printf("\nTotal gain with %d transactions:\n", n_transactions);
  logGain(log_debug, total_start, total_end, gain_percenteage);
  return gain_percenteage;
}

void singleSimulation(int n_transactions, float interest, bool is_refill, bool is_refill_failure, bool log_debug) {
  bool save_json_market_trader = true;
  bool save_json_transactions = true;
  bool save_json_simulations = false;

  simulation(interest, n_transactions, log_debug, save_json_simulations, save_json_market_trader, save_json_transactions, is_refill, is_refill_failure);
}

void multipleSimulations(int n_transactions_max, float interest_max, bool is_refill, bool is_refill_failure) {
  bool log_debug = false;
  bool save_json_market_trader = false;
  bool save_json_transactions = false;
  bool save_json_simulations = true;

  //rate of interest at each transaction
  float interest_start = 0.00;
  float interest_end = interest_max;
  float interest_step = 0.005;

  for(float interest = interest_start; interest <= interest_end; interest += interest_step) {
    //number of random transaction in a simulation
    int n_transactions_start = 10;
    int n_transactions_end = n_transactions_max;
    int n_transactions_step = 10;
    float mean_gain_percenteage[(n_transactions_end - n_transactions_start) / n_transactions_step];
    
    for(int n_transactions = n_transactions_start; n_transactions <= n_transactions_end; n_transactions += n_transactions_step) {
      //number of random simulations done each with n transaction (useful to get a mean value)
      int n_simulations = 10000;
      float simulation_gain_percentages[n_simulations];

      for (int i=0; i<n_simulations; i++) {
        float gain_percenteage = simulation(interest, n_transactions, log_debug, save_json_simulations, save_json_market_trader, save_json_transactions, is_refill, is_refill_failure);
        simulation_gain_percentages[i] = gain_percenteage;
      }
      float gain_percentages_mean = mean(simulation_gain_percentages, n_simulations);
      
      JsonSimulation(save_json_simulations, interest, n_transactions, gain_percentages_mean);

      if (n_transactions%100 == 0) 
        printf("interest: %.3f, n_transactions: %d \n", interest, n_transactions);
    }
  }
}

// opens the file and initialize it as json
void initializeFile(FILE** f, char* filename, char* field_name) {
  *f = fopen(filename, "w+");
  if (!*f) {
    printf("Error creating file %s\n", filename);
    exit(1);
  }
  fprintf(*f, "{\"%s\": [\n", field_name);
}

// close the file and concludes it as json
void closeFile(FILE** f) {
  fprintf(*f, "\n]}");
  fclose(*f);
}

// * args: {is_single_simulation} {n_transactions/n_transactions_max} {interest/interest_max} {is_refill} {is_refill_failure} {log_debug} (unused if !is_single_simulation)
int main(int argc, char** argv) {
  srand(time(0));

  if (argc < 7) {
    printf("use the arguments\n"); exit(1);
  }
  bool is_single_simulation = atoi(argv[1]);
  int n_transactions = atoi(argv[2]);
  float interest = atof(argv[3]);
  bool is_refill = atoi(argv[4]);
  bool is_refill_failure = atoi(argv[5]);
  bool log_debug = atoi(argv[6]);
  printf("is_single_simulation: %d\n", is_single_simulation);
  printf("n_transactions: %d\n", n_transactions);
  printf("interest: %.3f\n", interest);
  printf("is_refill: %d\n", is_refill);
  printf("is_refill_failure: %d\n", is_refill_failure);
  printf("log_debug: %d\n", log_debug);
  printf("\n");

  if (is_single_simulation) {
    initializeFile(&market_json_file, (char*)"single_sim_market.json", (char*)"market");
    initializeFile(&trader_json_file, (char*)"single_sim_trader.json", (char*)"trader");
    initializeFile(&transactions_json_file, (char*)"single_sim_transactions.json", (char*)"transactions");
    singleSimulation(n_transactions, interest, is_refill, is_refill_failure, log_debug);
    closeFile(&market_json_file);
    closeFile(&trader_json_file);
    closeFile(&transactions_json_file);
  }
  else {
    initializeFile(&simulations_json_file, (char*)"simulations.json", (char*)"simulations");
    multipleSimulations(n_transactions, interest, is_refill, is_refill_failure);
    closeFile(&simulations_json_file);
  }
  printf("\nsimulation finished\n\n");
}

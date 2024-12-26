# The price formula

If we used a simple proportion where good_market : euro_market = good_of_the_transaction : euro_to_pay, the formula for the euro_to_pay would be 
```
  (euro_market * good_market)
-------------------------------
  good_of_the_transaction
```

However, this formula is easily exploitable by the trader. So instead, we use not the current good_market but the future good_market that the market would have after the transaction. So the formula would be 
```
      (euro_market * good_market)
-------------------------------------------
  (good_of_the_transaction +- good_market)
```

Finally, we multiply this value by the interest (like 101%).

This formula has some interesting properties that we will see when simulating the usage.

# Single Simulation

We simulate n transactions that can randomly be either "buy" or "sell" with a random quantity of a random good. The transaction may fail.

In the graph we show the quantity exchanged and the euro paid for the transaction, as well as the market gain from it. You can notice how sometimes the gain is negative: the market is loosing from the transaction.

> The gain is calculated by converting all the market goods in default_currency before and after the transaction and using the difference between this two values in percenteage

The last two graphs are for showing the wallets of the market and the trader.

You can see how the market is able to drain the trader when the number of transactions is really big: our market is able, in the end, to always gain, even when loosing some money in some transactions.

(Disclaimer: all single simulations shown in the images are done with an interest rate of 1% on each transaction, the value used in the code itself and with using the market refill, which is explained later)

# Multiple Simulations

We can demostrate this by doing a lot of simulations. we cycle through a variable n_transactions from 5 to a certain limit and for each of of this number we execute 10'000 simulations and save the mean market gain from it. The graph shows how, for different interest rate, the more transactions our market does the more it gains globally, as we can see how each graph is ascending.

# Market refill strategy

The refill strategy is very simple: if a good is below a certain threshold, we try to refill it by exchanging the good we have most abundant, with a conversion rate calculated by using the default exchange rates and then applying a tax of 25% to be balanced

> Since every transaction must be an exchange between two goods, and we already demostrated how our market always gains the more transactions it makes, we are sure that there is always a good we have in abundance and that we can exchange.

Here we can see the power of our formula: by refilling the goods we have finished, our market gains are even greater, of almost 20% more!

## Market shortage

Every time we request a refill, there is the possibility that the good is shortaged and it can't be refilled for 100 days. We can see how this influence the market gain, but only slightly in a stocastic enviroment

> In the graphs, you can see how the Y scale of _simulations\_1k\_transactions\_w\_refill.png_ is a little be lower than _simulations\_1k\_transactions\_no\_refill\_failure.png_, meaning the maximum gain of the latter is a little be bigger than the one of the former

# Why our market

There are two main advantages of using our market:

- As you have seen, there are some transactions which are not advantageous for our market. Maybe you can find a way to exploit them :P

- Our market is sure to gain from every random transaction, in the long run: that means you can find an exploit in another market and use our to launder the money you made: both of us will gain from it
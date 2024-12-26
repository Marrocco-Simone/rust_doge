import json
import matplotlib.pyplot as plt

#measure unit GOOD/EUR
EUR_EUR_EXCHANGE_RATE = 1
EUR_USD_EXCHANGE_RATE = 1.03576
EUR_YEN_EXCHANGE_RATE = 143.615
EUR_YUAN_EXCHANGE_RATE = 7.3599

### transactions
filename = 'single_sim_transactions.json'
f = open(filename)
data = json.load(f)

transactions = data['transactions']
print(f"gathered {len(transactions)} points")

def get_quantity_moved(transaction):
  quantity = transaction['quantity']
  good_type  = transaction['good_type']
  if good_type == 'eur':  quantity /= EUR_EUR_EXCHANGE_RATE
  if good_type == 'usd':  quantity /= EUR_USD_EXCHANGE_RATE
  if good_type == 'yen':  quantity /= EUR_YEN_EXCHANGE_RATE
  if good_type == 'yuan': quantity /= EUR_YUAN_EXCHANGE_RATE
  return quantity
 
quantity = list(map(get_quantity_moved, transactions))
gain_percenteage = list(map(lambda t: t['gain_percenteage'], transactions))
euros_payed  = list(map(lambda t: t['euros_payed'], transactions))

f.close()

n_transactions = range(0, len(transactions))

fig, (axs_quantity, axs_gain, axs_euros_payed, axs_market, axs_trader) = plt.subplots(5)

axs_quantity.plot(n_transactions, quantity)
axs_quantity.set_xlabel("n transactions")
axs_quantity.set_ylabel("quantity exchanged")

axs_gain.plot(n_transactions, gain_percenteage)
axs_gain.set_xlabel("n transactions")
axs_gain.set_ylabel("gain percenteage")

axs_euros_payed.plot(n_transactions, euros_payed)
axs_euros_payed.set_xlabel("n transactions")
axs_euros_payed.set_ylabel("euros payed")

### market
filename = 'single_sim_market.json'
f = open(filename)
data = json.load(f)

market = data['market']
print(f"gathered {len(market)} points")
 
index_market = []
eur_market = []
usd_market = []
yen_market = []
yuan_market = []

for m in market:
  index_market.append(m['index'])
  eur_market.append(m['eur']/EUR_EUR_EXCHANGE_RATE)
  usd_market.append(m['usd']/EUR_USD_EXCHANGE_RATE)
  yen_market.append(m['yen']/EUR_YEN_EXCHANGE_RATE)
  yuan_market.append(m['yuan']/EUR_YUAN_EXCHANGE_RATE)

f.close()

axs_market.plot(index_market, eur_market, label = f"eur")
axs_market.plot(index_market, usd_market, label = f"usd")
axs_market.plot(index_market, yen_market, label = f"yen")
axs_market.plot(index_market, yuan_market, label = f"yuan")

axs_market.set_xlabel('time')
axs_market.set_ylabel('market goods in eur')

### trader
filename = 'single_sim_trader.json'
f = open(filename)
data = json.load(f)

trader = data['trader']
print(f"gathered {len(trader)} points")
 
index_trader = []
eur_trader = []
usd_trader = []
yen_trader = []
yuan_trader = []

for t in trader:
  index_trader.append(t['index'])
  eur_trader.append(t['eur']/EUR_EUR_EXCHANGE_RATE)
  usd_trader.append(t['usd']/EUR_USD_EXCHANGE_RATE)
  yen_trader.append(t['yen']/EUR_YEN_EXCHANGE_RATE)
  yuan_trader.append(t['yuan']/EUR_YUAN_EXCHANGE_RATE)

f.close()

axs_trader.plot(index_trader, eur_trader, label = f"eur")
axs_trader.plot(index_trader, usd_trader, label = f"usd")
axs_trader.plot(index_trader, yen_trader, label = f"yen")
axs_trader.plot(index_trader, yuan_trader, label = f"yuan")

axs_trader.set_xlabel('time')
axs_trader.set_ylabel('trader goods in eur')

plt.legend()
plt.show()
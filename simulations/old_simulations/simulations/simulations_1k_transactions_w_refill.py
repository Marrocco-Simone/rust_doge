import json
import matplotlib.pyplot as plt

filename = 'simulations_1k_transactions_w_refill.json'
f = open(filename)
data = json.load(f)

simulations = data['simulations']
print(f"gathered {len(simulations)} points")
 
interests = {}

for simulation in simulations:
  inter = simulation['interest']
  if inter not in interests:
    interests[inter] = {"x": [], "y": []}
  interests[inter]["x"].append(simulation['n_transactions'])
  interests[inter]["y"].append(simulation['gain_percentages_mean'])

f.close()

for key, value in interests.items():
  x = value['x']
  y = value['y']
  plt.plot(x, y, label = f"interest= {key}")

plt.xlabel('n_transactions')
plt.ylabel('mean gain percentage')
plt.title('Market gain doing n transactions with a certain interest, with market refill and refill failure')

plt.legend()
plt.show()
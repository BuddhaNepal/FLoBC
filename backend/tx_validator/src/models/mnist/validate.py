
#In[1]
import sys
import matplotlib.pyplot as plt
import pandas as pd
import numpy as np
import tensorflow as tf

# %%
################################
# Formatted print back to node
################################
def send_to_node(update_vector):
    if len(update_vector) == 0:
        print("VECTOR[]ENDVECTOR")
    else:
        print("VECTOR[", end='')
        for i in range(len(update_vector) - 1):
            print(update_vector[i], end=',')
        print(update_vector[-1], end='')
        print("]ENDVECTOR")
# %%
################################
# Reading dataframe
################################
def read_input(data_dir):
    #if len(sys.argv) < 2:
    #    raise Exception('No dataset path found')

    df = pd.read_csv(data_dir)
    # df = pd.read_csv("resized_test.csv")
    if len(df) == 0:
        raise Exception('Empty dataset')
    return df

# %%
################################
# Reshaping input
################################
def reshapeData(data_dir):
  df = read_input(data_dir)
  label = df.iloc[:, 0]
  label = label.to_numpy()
  df = df.drop(df.columns[0], axis = 1)
  df = df.values.reshape(df.shape[0], 20, 20)

  #df = df.reshape(df.shape[0], 20, 20, 1)
  # Making sure that the values are float so that we can get decimal points after division
  df = df.astype('float32')
  # Normalizing the RGB codes by dividing it to the max RGB value.
  df /= 255
  return df, label

# %%
def createModel():
  model = tf.keras.models.Sequential([
    tf.keras.layers.Flatten(input_shape=(20, 20)),
    # tf.keras.layers.Dense(10, activation='relu'),
    # tf.keras.layers.Dropout(0.2),
    tf.keras.layers.Dense(10)
  ])
  loss_fn = tf.keras.losses.SparseCategoricalCrossentropy(from_logits=True)
  model.compile(optimizer='adam',
                loss=loss_fn,
                metrics=['accuracy'])
  return model

# %%
def evaluateModel(model, data_test, label_test):
  results = model.evaluate(data_test, label_test, verbose=2)
  # Return accuracy
  return results[1]

  if(results[1] > 0.7):
    return True
  else:
    return False

# %%
def flattenWeights(model):
  arr = np.array(model.get_weights())
  for i in range (0, len(arr)):
          arr[i] = arr[i].flatten()

  arr = np.concatenate(arr)
  flat_model = arr.tolist()
  return flat_model

# %%
def rebuildModel(flat_model):
  new_model = createModel()
  start = 0
  for i in range (0, len(new_model.layers)):
      bound = np.array(new_model.layers[i].get_weights()).size
      weights = []
      for j in range (0, bound):
        size = (new_model.layers[i].get_weights()[j]).size
        arr = np.array(flat_model[start:start+size])
        arr = arr.reshape(new_model.layers[i].get_weights()[j].shape)
        weights.append(arr)
        start += size
      if (bound > 0):
        new_model.layers[i].set_weights(weights)
  return new_model
# %%

################################
# Validation score
################################
def compute_validation_score(flat_model, data_dir):
  data_test, label_test = reshapeData(data_dir)
  # list = [0.5] * 4010
  model = rebuildModel(flat_model)
  result = evaluateModel(model, data_test, label_test)
  print(result)
  return result
# %%
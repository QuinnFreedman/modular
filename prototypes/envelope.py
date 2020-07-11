import math
import matplotlib.pyplot as plt

def exp(x, degree):
    base = math.e
    range_start = -2
    range_end = 2

    exp1 = base ** (range_start * degree)
    exp2 = base ** (range_end * degree)
    
    return (base ** (degree * x) - exp1) / (exp2 - exp1)



plt.plot([exp(x/100, 2) for x in range(100)])
plt.show()

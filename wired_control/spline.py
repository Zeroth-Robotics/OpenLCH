
def GetHermiteY(t, x0, x1, y0, y1, m0, m1):
    return (h_00(t) * y0) + (h_10(t) * (x1 - x0) * m0) + (h_01(t) * y1) + (h_11(t) * (x1 - x0) * m1)

def GetHermiteY_1(t, y0, y1, m0, m1):
    return (h_00(t) * y0) + (h_10(t) * m0) + (h_01(t) * y1) + (h_11(t) * m1)

def GetSimpleHermiteY(t, y0, y1):
    return (h_00(t) * y0) + (h_01(t) * y1)
    
def GetTangent(x1, y1, x2, y2):
    diff = x2 - x1
    if diff == 0: return 0
    return (y2 - y1) / diff

# Hermite functions
def h_00(t): return (2 * pow(t, 3)) - (3 * pow(t, 2)) + 1
def h_10(t): return pow(t, 3) - (2 * pow(t, 2)) + t
def h_01(t): return (3 * pow(t, 2)) - (2 * pow(t, 3))
def h_11(t): return pow(t, 3) - pow(t, 2)

class Spline:
    def __init__(self, x_points, y_points):
        self.x = x_points
        self.y = y_points

        self.m0 = GetTangent(x[0], y[0], x[2], y[2]) # Get tangent between points 0 and 2
        self.m1 = GetTangent(x[1], y[1], x[3], y[3]) # Get tangent between points 1 and 3
        
    def GetY(self, x):  # Get Y between points 1 and 2
        if x < self.x[1]: return self.y[0];
        if x > self.x[2]: return self.y[3];

        frame = self.x[2] - self.x[1]
        if frame == 0:
            return 0 # Error (bad points)

        # Get location of x between points 1 and 2 in fraction (percent)
        t = (x - self.x[1]) / frame;
        return GetHermiteY(t, self.x[1], self.x[2], self.y[1], self.y[2], self.m0, self.m1);

if __name__ == '__main__':
    '''
    x = [10, 20, 30, 40]
    y = [100, 5, 10, 20]
    s = Spline(x, y)
    for i in range(10, 41):
        print(i, s.GetY(i))
    '''
    for i in range(20, 31):
        t = (i-20) / 10.0
        #print(i, GetSimpleHermiteY(t, 5, 10))
        print(i, GetHermiteY_1(t, 5, 10, 0, 100))
  

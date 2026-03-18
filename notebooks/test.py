import marimo

__generated_with = "0.13.0"
app = marimo.App()


@app.cell
def _():
    import numpy as np
    import matplotlib.pyplot as plt
    return np, plt


@app.cell
def _(np, plt):
    x = np.linspace(0, 2 * np.pi, 200)
    y = np.sin(x)

    fig, ax = plt.subplots()
    ax.plot(x, y)
    ax.set_title("Hello from marimo!")
    ax.set_xlabel("x")
    ax.set_ylabel("sin(x)")
    fig
    return


if __name__ == "__main__":
    app.run()

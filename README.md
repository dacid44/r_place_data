# r/place Heatmap

I thought it would be interesting to plot out the pixels placed during [Reddit's r/place 2022 experiment](https://www.reddit.com/r/place/)
onto a heatmap and see what areas of the map were the most contested, and if any pixels never got painted over at all.

For anyone who may not know, r/place was an experiment that Reddit conducted for April Fools' day in 2017, and again in 2022. Anyone in the world (with a Reddit account) could place a pixel with a color of their choice onto the single
global canvas, every few minutes. This resulted in clans and teams forming, and significant strategy and diplomacy
taking place as well, as it was essentially a massive internet turf war.

The core parts of this project (the parsing of the dataset and tallying of the frequency data) were written in Rust,
using PyO3 to expose them to Python. This allowed me to use a Jupyter notebook and Matplotlib to easily play with
different methods of visualizing the data.

Though the dataset is too big to include in the repo, it can be obtained [here](https://www.reddit.com/r/place/comments/txvk2d/rplace_datasets_april_fools_2022/).

Here is the final result! I've included two different versions. They show the same data, but I found different colormaps
help to show different parts of the canvas better. The first shows pixels with more placements in red, and less closer
to yellow, with pixels never placed on transparent, and the second shows more contested pixels darker, with pixels with
no placements in red. I'll also include an image of the final r/place canvas below for reference.

![heatmap](results/heatmap.png)

![heatmap-bw](results/heatmap-bw.png)

![final-place](final-place.png)

This was a super fun project to work on, and I learned a lot about the internals of both Rust and Python through working
on it! There was also a lot of optimization that needed to be done to parse the data on my own PC in a reasonable amount
of time. (Lesson learned: regexes are super useful, but also pretty slow in comparison to less dynamic parsing tools.)

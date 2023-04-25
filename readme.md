# Inventory Server

This is a simple inventory server to track products for small hobby businesses. The dataset it maintains is:
- Product Name: a descritive title (ie. "Umbrella Corp Vinyl Sticker")
- SKU: screaming snake case SKU (ie. UMBC_VINYL)
- Sold: track the amount sold
- Stock: track the amount currently in inventory
- Created: date in which the product was added to the database
- Retired: boolean flag for preserving data while excluding them from weekly reports

## Implementation

Deviation from this is possible, but might require modifing the code. Future versions might implement feature
choices or other means to modify functionality without having to make changes to the source code. For now,
this software is an open source version of my own use case.

### Setup
Since the current version of this remake does not yet implement any email reporting, it is acceptable to
run the program only when needed. This might make for a more user-friendly experience in the design if I
implement an offloaded reporting system that doesn't rely on a 24/7 runtime. Just a thing to consider at
the moment.

```Products.json``` is the "database" file. There is an example product preloaded to give you an immediate
hands on approach to initializing your database if that is easier. Delete this exact entry though as json
has no comment-out-code option.

### Running for the first time
Running ```cargo run``` will compile and run the debug version and use the ```Products.json``` file in the
/src directory. Better is to compile the release with ```cargo build --release``` and then move or copy
```target/release/inventory_scanner.exe``` and ```src/Products.json``` into a folder together.

Once the program opens, there will be no preloaded products so you will have to enter them with the sequence
```Q+[SKU](Title)#```. Let's look at a couple examples.

    Q+[UMBC_VINYL](Umbrella Corp Vinyl Sticker)0
    Q+[MAOWS](Mouse Cat Toy)5
    Q+[SKOOMA_BRN](Do You Want Some Skooma Patch - Brown)2

Each line should be entered one by one, multi-add is not a current feature. This though will create 3 different
products in the json file used as the database. It will then initialize the Umbrella Corp sticker with 0 stock.
The cat toy will have 5 immediately in stock, and 2 of the patches. All of these then also get date tagged with
today's date. Date's don't have any direct use in this software, but can be useful to have for other business
reasons elsewhere.

### Stocking and Selling
The main interaction with the software is to either stock or sell products. This is handled by a simple text
command ```SKU*#```. Let's look at a couple examples.

    UMBC_VINYL*2
    MAOWS*-1
    SKOOMA_BRN*0

Again, each line should be entered one by one, multiline commands are not a current feature. The first example
is a positive numbered SKU, so we are increasing the stocked quantity. This will add 2 to Stock on that SKU.
There is no change to Sell. Next, we have -1 to sell 1 MAOWS. Stock will drop to 4 and Sell up to 1. For
SKOOMA_BRN, I have added a feature that I personally use in the event of gifting. 0 specifically will reduce
the Stock amount by 1, but wont increase Sell like a negative number would. This has a caviat of misrepresenting
the true manufactured quantities, but does support tracking populatiry by sales. Meaning, if I make 20 of
something that never sells, and I give them all away, it looks as though I never made them. Not great, but I am
more concerned with not making it look like I successfully sold all 20 when truely no one intentionall bought
them.

### Retire, Restore, and Inspect
These are helper functions to give you the information you need when you need it. Let's look at a couple examples.

    retire:UMBC_VINYL
    restore:MAOWS
    inspect:SKOOMA_BRN

Still, each line should be entered one by one, no multiline here yet. The first example will change the Umbrella
Corp sticker to retired. That means it won't show up on weekly report tables. You can still sell or stock it.
Next is restore. Rather than have the feature be ```retire:SKU``` to toggle retired on and off, retire and
restore respectively set the retire flag to ```true``` or ```false```. Lastly is inspect. When you stock or sell
things, by default, it will echo out sold and stock counts. Inspect is a way to easily poll that information
without making such an action.

## Email Report

This version does not yet have email reporting implemented.

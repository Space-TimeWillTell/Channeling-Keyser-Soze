# Extract high quality pages from two-cards-per-page PDF

echo "Extracting max quality pages"

for i in `seq 0 10`; do
    echo "Extracting max quality page $i (temporary file)";
    convert -density 300 assets/Deck-two-per-page.pdf[$i] -quality 90 -trim /tmp/Deck-two-per-page$i.jpg;
done


# Extract high quality horizontal cards
# (not the back)

echo "Extracting max quality horizontal cards"

for i in `seq 0 10`; do
    echo Extracting max quality horizontal card $((2 * $i));
    convert /tmp/Deck-two-per-page$i.jpg[0x2117+0+0] -trim /tmp/horizontal_card_$((2 * $i)).jpg;
    echo Extracting max quality horizontal card $((2 * $i + 1));
    convert /tmp/Deck-two-per-page$i.jpg[0x2117+0+2117] -trim /tmp/horizontal_card_$((2 * $i + 1)).jpg;
done

echo "Extracting up and down cards"

# Extract small horizontal cards

for i in `seq 0 21`; do
    echo "Extracting horizontal thumbnail $i (asset)";
    convert /tmp/horizontal_card_$i.jpg -resize 12.5% assets/small_horizontal_card_$i.png;

    echo "Extracting high quality up card $i (asset)";
    convert /tmp/horizontal_card_$i.jpg -resize 25% -rotate 90 assets/card_$i.png;
    echo "Extracting high quality down card $i (asset)";
    convert /tmp/horizontal_card_$i.jpg -resize 25% -rotate -90 assets/reversed_card_$i.png;

    echo "Extracting thumbnail up card $i (asset)";
    convert /tmp/horizontal_card_$i.jpg -resize 12.5% -rotate 90 assets/small_card_$i.png;
    echo "Extracting thumbnail down card $i (asset)";
    convert /tmp/horizontal_card_$i.jpg -resize 12.5% -rotate -90 assets/small_reversed_card_$i.png;
done

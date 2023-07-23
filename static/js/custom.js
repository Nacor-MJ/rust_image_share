sheet = document.getElementById("sheet");
currentLine = 0;

addLineToSheet();


function addLineToSheet() {
    line = document.createElement("span");
    line.setAttribute("class", "line");
    line.setAttribute("id", String(currentLine));
    currentLine += 1;
    sheet.appendChild(line);

    addStuffToLine(line);
    line.children[0].innerHTML = "chords";
    line.children[1].innerHTML = "lyrics";
    return line;
}

function addStuffToLine(line) {
    chords = document.createElement("span");
    chords.setAttribute("class", "chord");
    chords.setAttribute("contenteditable", "true");
    line.appendChild(chords);
    
    lyrics = document.createElement("span");
    lyrics.setAttribute("class", "lyrics");
    lyrics.setAttribute("contenteditable", "true");
    line.appendChild(lyrics);
}

function removeLine() {
    sheet.removeChild(sheet.lastChild);
}

function saveAs() {
    const dataToSend = document.getElementById("sheet").innerHTML;

    const fileName = prompt("Enter the song name: ");

    fetch("/saving-html", {
    method: "POST",
    headers: {
        "Content-Type": "text/html",
    },
    body: fileName + " \n" + dataToSend,
    })
    .then((response) => response.text())

    .catch((error) => {
        console.error("Error:", error);
    });
}
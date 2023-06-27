window.onload = function() {
    loadZpevnik(localPage)
}
function loadZpevnik(currentPage) {
    updateSyncState();
    const formatedPageNumber = currentPage.toString().padStart(3, '0');
    
    page = document.getElementById("page");
    page.src = "/static/images/LabeZpevnik/LabeZpevnik-" + formatedPageNumber + ".png";

    const pageNumberElement = document.getElementById("page-number");
    pageNumberElement.textContent = "Page number: " + currentPage;
    if (currentPage == 420 || currentPage == 69) {
        pageNumberElement.textContent += "-Nice!";
    };
}

const defaultPage = 5;
var globalPage = defaultPage;
var localPage = defaultPage;

function syncToGlobal() {
    fetch('/static/json/global-variable.json')
        .then(response => response.json())
        .then(data => {
            globalPage = data.number;
            console.log('Received number:', globalPage);
        })
        .catch(error => {
            console.error('Error:', error);
        });

    localPage = globalPage;
    loadZpevnik(localPage);
}

function updateSyncState() {
    if (localPage == globalPage) {
        document.getElementById("sync-state").textContent = "Synced";
    } else {
        document.getElementById("sync-state").textContent = "Not Synced";
    };
}

function jumpToPage(symbol='') {
    if (symbol === '+'){
        localPage -= (-1);
    } else if (symbol === '-') {
        localPage -= (1);
        if (localPage < 1) {
            localPage = 1;
        }
    } else if (!(symbol ==="boobs")){
        localPage = document.getElementById("newLocalPage").value;
    };
    loadZpevnik(localPage);
}

function move(direction) {
    jumpToPage(direction);
}
function sync() {
    syncToGlobal();
}

const eventSource = new EventSource('/sse');

eventSource.addEventListener('message', (event) => {
  const eventData = event.data;

  console.log('Received SSE event:', eventData);
});

eventSource.addEventListener('error', (error) => {
  console.error('Error in SSE connection:', error);
});


var textBox = document.getElementById("newLocalPage");
textBox.addEventListener("keypress", function(event) {
    if (event.key === "Enter") {
        event.preventDefault();
        jumpToPage(); 
    };
});
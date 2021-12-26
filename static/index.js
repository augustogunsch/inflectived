let polishSchemas = null;

$.ajax({
    url: '/static/schemas/polish.json',
    success: data => polishSchemas = data
});

let searchBar = $('#search-bar');

searchBar.autocomplete({
    source: (request, response) => {
        $.ajax({
            url: '/langs/polish/words?like=' + request.term + '&limit=20&offset=0',
            success: data => response(data)
        })
    }
});

$('#search-form').on('submit', (e) => {
    e.preventDefault();

    let word = e.target[0].value

    getWord(word);
});

function getWord(word) {
    $.ajax({
        url: '/langs/polish/words/' + word,

        success: (data) => {
            $('#ajax-content').html(generateHtml(data))
            searchBar.autocomplete('close');
        },

        error: err => console.error(err)
    })
}

function getCells(forms, tags) {
    if(tags.length === 0)
        return undefined;

    let cells = forms.filter(form =>
        tags.every(value => form.tags.includes(value))
    );

    cells.forEach(cell =>
        cell.used = true
    );

    if(cells.length === 0)
        return undefined;

    return cells;
}

function generateList(data) {
    let html = '<ul>';
    data.forEach(cell =>
        html += `<li><strong>${cell.form}</strong> - ${cell.tags.join(', ')}</li>`
    );
    html += '</ul>';

    return html;
}

function generateTable(schemas, pos, forms) {
    let schema = schemas.find(schema => schema.pos === pos);

    // No schema was provided by the server - fallback to a list
    if(!schema)
        return generateList(forms);

    let html = '<table>';

    schema.rows.forEach(row => {
        html += '<tr>';
        row.forEach(cell => {
            if('display' in cell) {
                html += `<th colspan="${cell.colspan}" rowspan="${cell.rowspan}">${cell.display}</th>`;
            } else {
                let cells = getCells(forms, cell.tags);
                let content = cells ? cells.map(cell => cell.form).join(', <br>') : '-';
                html += `<td colspan="${cell.colspan}" rowspan="${cell.rowspan}">${content}</td>`;
            }
        });
        html += '</tr>';
    });

    html += '</table>';

    let unusedCells = forms.filter(cell => !cell.used);

    if(schema.ignoreUnused) {
        unusedCells = unusedCells.filter(cell =>
            !schema.ignoreUnused.map(tags => tags.every(tag => cell.tags.includes(tag)))
        );
    }

    if(unusedCells.length > 0) {
        html += '<h3>Other</h3>';
        html += generateList(unusedCells);
    }

    return html;
}

function generateHtml(data) {
    let html = '';

    data.forEach(entry => {
        html += `<h1>${entry.word} <span class="pos">(${entry.pos})</span></h1>`
        if('senses' in entry) {
            if('tags' in entry.senses[0]) {
                html += '<div class="tags">'
                html += entry.senses[0].tags.map(tag => `<span class="tag">${tag}</span>`).join(', ');
                html += '</div>'
            }

            html += '<h2>Senses</h2>';

            html += '<ol>';
            entry.senses.forEach(sense => {
                html += '<li>'

                if('form_of' in sense) {
                    let word = sense.form_of[0].word;
                    html += sense.glosses[0].replace(new RegExp(`of ${word}$`), '');
                    html += ` of <a onclick="getWord('${word}')">${word}</a>`;
                } else {
                    html += sense.glosses[0];
                }

                html += '</li>';
            })
            html += '</ol>';
        }

        if('forms' in entry) {
            if(entry.pos === 'verb') {
                let conjugation = entry.forms.filter(form =>
                    'source' in form && form.source === 'Conjugation');

                if(conjugation.length > 0) {
                    html += '<h2>Conjugation</h2>';

                    html += generateTable(polishSchemas, entry.pos, conjugation);
                }
            } else {
                let declension = entry.forms.filter(form =>
                    'source' in form && form.source === 'Declension');

                if(declension.length > 0) {
                    html += '<h2>Declension</h2>';

                    html += generateTable(polishSchemas, entry.pos, declension);
                }
            }
        }
    });

    return html;
}

function listForms(data) {
    html += '<ul>';
    data.forms.map(form => {
        html += `<li>${form.form}</li>`;
    })
    html += '</ul>';
}

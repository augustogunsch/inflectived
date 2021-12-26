$(document).ready(() => {
    let polishSchemas = null;

    $.ajax({
        url: '/static/schemas/polish.json',
        success: data => {
            polishSchemas = data
            if(window.location.hash) {
                getWord();
            }
        }
    });

    window.onhashchange = () => {
        getWord();
    };

    let searchBar = $('#search-bar');

    searchBar.autocomplete({
        source: (request, response) => {
            $.ajax({
                url: '/langs/polish/words?like=' + request.term + '&limit=20&offset=0',
                success: data => response(data)
            })
        }
    });

    $('#search-bar').on('focus', e => {
        setTimeout(() => e.currentTarget.select(), 100);
    });

    $('#search-form').on('submit', (e) => {
        e.preventDefault();

        let word = e.target[0].value

        window.location.hash = `#${word}`;
    });

    function getWord() {
        let word = window.location.hash.replace('#', '');
        $.ajax({
            url: '/langs/polish/words/' + word,

            success: (data) => {
                $('#ajax-content').html(generateHtml(word, data))
                searchBar.autocomplete('close');
                window.scrollTo(0, 0);
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
        let schema = schemas.find(schema => schema.pos.includes(pos));

        // No schema was provided by the server - fallback to a list
        if(!schema)
            return generateList(forms);

        let html = '<div class="table-responsive">';
        html += '<table class="table table-sm table-bordered border-dark text-center align-middle">';

        schema.rows.forEach(row => {
            html += '<tr>';
            row.forEach(cell => {
                if('display' in cell) {
                    html += `<th class="table-light border-dark" colspan="${cell.colspan}" rowspan="${cell.rowspan}">${cell.display}</th>`;
                } else {
                    let cells = getCells(forms, cell.tags);
                    let content = cells ? cells.map(cell => cell.form).join(', <br>') : '-';
                    html += `<td colspan="${cell.colspan}" rowspan="${cell.rowspan}">${content}</td>`;
                }
            });
            html += '</tr>';
        });

        html += '</table>';
        html += '</div>';

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

    function generateHtml(word, data) {
        let html = '';

        if(data.length === 0) {
            html += `<h1>Not found: <mark>${word}</mark></h1>`;
        } else {
            data.forEach(entry => {
                html += `<h1>${entry.word} <span class="pos">(${entry.pos})</span></h1>`
                if('senses' in entry) {
                    let tags = [];
                    entry.senses.forEach(sense => {
                        if('tags' in sense) {
                            tags.push(...sense.tags);
                        }
                    });

                    if(tags.length > 0) {
                        tags = [...new Set(tags)];
                        html += '<div class="tags">Tags: '
                        html += tags.map(tag => `<mark>${tag}</mark>`).join(', ')
                        html += '</div>'
                    }

                    html += '<h2>Senses</h2>';

                    html += '<ol>';
                    entry.senses.forEach(sense => {
                        html += '<li>'

                        if('form_of' in sense) {
                            let word = sense.form_of[0].word;
                            html += sense.glosses[0].replace(new RegExp(`of ${word}$`), '');
                            html += ` of <a href="#${word}" class="link-primary">${word}</a>`;
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
        }

        return html;
    }
});

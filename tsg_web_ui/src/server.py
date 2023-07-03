#!/usr/bin/env python3

import tsg_web_ui
import json
import os
import csv
import pandas as pd
import tempfile
import copy

from flask import Flask, render_template, request, url_for, flash, redirect, send_file

TEMPLATES_AUTO_RELOAD = True

info = None
with open(os.path.abspath('info.json')) as inf_f:
    info = json.load(inf_f)

app = Flask(__name__)

app.secret_key = "CHANGEME"

@app.route("/gen_ts", methods=["POST"])
def gen_ts():
    print(request.files)
    if 'csv' not in request.files:
        flash("Missing CSV file")
        return redirect(url_for('index'))

    csv_f = request.files['csv']
    csv_read = pd.read_csv(csv_f.stream)
    csv_read = csv_read.rename(columns=lambda x: x.lower())
    # entries for info
    entries = csv_read.to_dict('records')

    name = request.values.get('name')
    address = request.values.get('address')

    info_d = copy.deepcopy(info)
    for i in range(0, len(info_d['pos_data'])):
        dn = info_d['pos_data'][i]['data_name']
        if dn == 'name':
            info_d['pos_data'][i]['data_value']['value'] = name
        elif dn == 'address':
            info_d['pos_data'][i]['data_value']['value'] = address
    info_d['entries'] = entries

    outfile_path = os.path.join(tempfile.mkdtemp(), 'output.pdf')
    tsg_web_ui.gen_ts(os.path.abspath("timesheet.pdf"), json.dumps(info_d), "pdf", outfile_path)

    # TODO delete old pdfs
    # print(request.values.get('values'))

    return send_file(outfile_path)


@app.route("/")
def index():
    return render_template("index.html")

if __name__ == "__main__":
    app.run()

# tsg_web_ui.gen_ts(os.path.abspath("../timesheet_gen/timesheet.pdf"), json.dumps(info), 'pdf', 'output.pdf')
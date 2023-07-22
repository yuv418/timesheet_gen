FROM rust:1-bookworm
RUN apt update && apt install -y python3-pip build-essential python3-dev patchelf python3-poetry gunicorn fonts-liberation

WORKDIR /app
COPY ./tsg_web_ui /app
COPY ./timesheet_gen /timesheet_gen

RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall -y maturin

RUN poetry install
RUN apt install -y libpoppler-glib-dev


RUN poetry run maturin develop --release

CMD ["poetry", "run", "gunicorn", "-b", "0.0.0.0:8000", "src.server:app"]

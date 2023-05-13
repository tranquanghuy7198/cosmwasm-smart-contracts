FROM cosmwasm/workspace-optimizer:0.12.11 as workspace

COPY . .

RUN optimize_workspace.sh .

FROM node:16.6.0-alpine

ARG RELEASE_VERSION
ARG RELEASE_COMMIT
ENV RELEASE_VERSION=$RELEASE_VERSION
ENV RELEASE_COMMIT=$RELEASE_COMMIT

WORKDIR /code

COPY --from=workspace /code/artifacts /code/artifacts
RUN echo ${RELEASE_COMMIT} > /code/artifacts/release.commit
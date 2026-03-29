import type { GetServerSideProps, NextPage } from 'next';

type Props = { id: number };

const TemplateDetails: NextPage<Props> = ({ id }) => {
  return <div>Template Details for ID: {id}</div>;
}

export const getServerSideProps: GetServerSideProps<Props> = async (ctx) => {
  const rawId = ctx.params?.id;

  // Validate and parse id
  if (Array.isArray(rawId)) {
    return { notFound: true };
  }

  const idNum = Number(rawId);

  if (!Number.isFinite(idNum)) {
    return { notFound: true };
  }

  return {
    props: { id: idNum },
  };
};

export default TemplateDetails;
